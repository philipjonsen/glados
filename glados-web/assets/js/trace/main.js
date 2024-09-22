// Creates a graph from a trace object and returns an SVG.
function createGraph (graphData, sortByNodeId = false) {
  const graph = ForceGraph(graphData, {
    nodeId: (d) => d.id,
    nodeGroup: (d) => d.group,
    nodeGroups: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
    nodeTitle: (d) => generateNodeMetadata(d),
    linkStrokeWidth: (l) => Math.sqrt(l.value),
    width: $('#graph').width(),
    height: $('#graph').height(),
    invalidation: null,
    contentId: graphData.contentId,
    sortByNodeId
  })

  return graph
}

const colors = {
  blue: 0,
  orange: 1,
  red: 2,
  green: 4,
  yellow: 5,
  brown: 8,
  gray: 9
}

// Converts json response to format expected by D3 ForceGraph:
// { nodes: [{ id, group }], links: [{ source_id, target_id, group }] }
// Group of nodes determines color, group of links determines thickness.
function createGraphData (trace) {
  if (Object.keys(trace).length === 0) {
    return {
      nodes: [{ id: 'local', group: colors.orange, durationMs: 0 }],
      links: []
    }
  }

  const successfulRoute = computeSuccessfulRoute(trace)

  console.log('Route:')
  console.log(successfulRoute)

  const metadata = {}
  Object.keys(trace.metadata).forEach((nodeId) => {
    const meta = trace.metadata[nodeId]
    const enr = meta.enr
    const decodedEnr = ENR.ENR.decodeTxt(enr).enr

    const ip = decodedEnr.ip || 'localhost' // Edge case for local node
    const port = decodedEnr.udp
    const distance = BigInt(meta.distance)
    const distanceLog2 = bigLog2(distance)
    const client = decodedEnr.client
    const radius = meta.radius

    metadata[nodeId] = {
      enr,
      ip,
      port,
      distance,
      distanceLog2,
      client,
      radius
    }
  })

  // Create nodes.
  const nodes = []
  const nodesSeen = []
  const responses = trace.responses
  Object.keys(responses).forEach((nodeId, _) => {
    const node = responses[nodeId]
    const durationMs = node.durationMs
    const respondedWith = node.respondedWith
    if (!Array.isArray(respondedWith)) {
      return
    }
    if (!nodesSeen.includes(nodeId)) {
      let group = 0
      if ('origin' in trace && trace.origin == nodeId) {
        group = colors.orange
      } else {
        if ('receivedFrom' in trace && trace.receivedFrom == nodeId) {
          group = colors.green
        } else if (respondedWith.length == 0) {
          group = colors.brown
        } else if (trace.cancelled.includes(nodeId)) {
          group = colors.yellow
        } else {
          group = colors.blue
        }
      }
      nodes.push({
        id: nodeId,
        group,
        durationMs,
        ...metadata[nodeId]
      })
      nodesSeen.push(nodeId)
    }
  })

  // Create links.
  const links = []
  Object.keys(responses).forEach((nodeIdSource, _) => {
    const node = responses[nodeIdSource]
    const respondedWith = node.respondedWith
    if (!Array.isArray(respondedWith)) {
      return
    }
    respondedWith.forEach((nodeIdTarget, _) => {
      if (!nodesSeen.includes(nodeIdTarget)) {
        let group = colors.gray

        if (trace.cancelled.includes(nodeIdTarget)) {
          group = colors.yellow
        }

        nodes.push({
          id: nodeIdTarget,
          group,
          ...metadata[nodeIdTarget]
        })
        nodesSeen.push(nodeIdTarget)
      }
      let value = 1
      if (
        successfulRoute.includes(nodeIdSource) &&
        successfulRoute.includes(nodeIdTarget)
      ) {
        value = 40
      }
      links.push({
        source: nodeIdSource,
        target: nodeIdTarget,
        value
      })
    })
  })
  const graph = {
    nodes,
    links,
    metadata: {
      nodesContacted: nodes.length - 1,
      nodesResponded: Object.keys(responses).length - 1
    },
    contentId:
      '0x' +
      trace.targetId.map((byte) => byte.toString(16).padStart(2, '0')).join('')
  }
  console.log(graph)
  return graph
}

// Returns a list of nodes in the route.
// Starts from the end (where the content was found) and finds the way back to the origin.
function computeSuccessfulRoute (trace) {
  if (!('origin' in trace && 'receivedFrom' in trace)) {
    return []
  }

  const origin = trace.origin
  let receivedFrom = trace.receivedFrom

  const route = []
  route.push(receivedFrom)
  const route_info = trace.responses
  while (receivedFrom != origin) {
    const previous_target = receivedFrom
    Object.keys(route_info).forEach((nodeId, _) => {
      const node = route_info[nodeId]
      const responses = node.respondedWith

      // Find the node that responded with the current target node.
      if (Array.isArray(responses) && responses.includes(receivedFrom)) {
        receivedFrom = nodeId
        route.push(receivedFrom)
      }
    })
    if (previous_target == receivedFrom) {
      // Did not progress, no route found.
      return []
    }
  }
  return route
}

// Generates a string to appear on hover-over of a node.
function generateNodeMetadata (node) {
  const durationMs = node.durationMs
  const client = node.client
  let metadata = `${node.id}\n`
  if (durationMs !== undefined) {
    metadata += `${durationMs} ms\n`
  }
  if (client !== undefined) {
    metadata += `${client}`
  }
  if (node.radius) {
    const radius_numerator = BigInt(node.radius)
    const radius_denominator = BigInt(
      '0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff'
    )
    const radius_percentage =
      (Number(radius_numerator.toString()) /
        Number(radius_denominator.toString())) *
      100
    metadata += `\nRadius Percent: ${radius_percentage.toFixed(2)}%`
  }
  return metadata
}

function generateTable (nodes) {
  $('#enr-table').empty()
  nodes.sort((a, b) =>
    a.distance < b.distance ? -1 : a.distance > b.distance ? 1 : 0
  )
  nodes.forEach((node, index) => {
    let nodeIdString = node.id
    nodeIdString =
      nodeIdString.substr(0, 6) +
      '...' +
      nodeIdString.substr(nodeIdString.length - 4, nodeIdString.length)

    const enr_shortened =
      node.enr.substr(5, 4) +
      '...' +
      node.id.substr(node.id.length - 5, node.id.length)

    const tr = document.createElement('tr')
    tr.innerHTML = `<th scope="row">${index + 1}</th>
            <td>${enr_shortened}</td>
            <td>${nodeIdString}</td>
            <td>${node.distanceLog2}</td>
            <td>${node.ip}:${node.port}</td>
            <td>${node.client === undefined ? '' : node.client}</td>
            <td>${node.radius === null ? 'unknown' : BigInt(node.distance) < BigInt(node.radius)}</td>`

    tr.addEventListener('mouseenter', () => {
      tr.style.backgroundColor = 'lightgray'
      highlightNode(node.id)
    })
    tr.addEventListener('mouseleave', () => {
      tr.style.backgroundColor = 'white'
      unHighlight()
    })
    tr.id = node.id.substring(4)

    $('#enr-table').append(tr)
  })
}

function highlightTableEntry (node) {
  unHighlight()
  const enr = node.target.__data__.id
  const enr_substr = enr.substring(4)
  const id_string = '#' + enr_substr
  $(id_string).css('background-color', 'lightgray')

  const element = document.getElementById(enr_substr)
  element.scrollIntoView({ block: 'nearest', behavior: 'auto' })

  highlightNode(enr)
}

function highlightNode (node) {
  d3.selectAll('g')
    .selectAll('circle')
    .filter((d) => d.id === node)
    .attr('r', function (node) {
      return 10
    })
}

function unHighlight () {
  d3.selectAll('g')
    .selectAll('circle')
    .attr('r', function (node) {
      return 5
    })
  $('tr').css('background-color', 'white')
}

function bigLog2 (num) {
  const one = BigInt(1)
  let ret = BigInt(0)

  while ((num >>= one)) ret++

  return ret
}

function supportsTrace (client) {
  (client === 'trin') | (client === 'fluffy')
}
