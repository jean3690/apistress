import type { TestPlan, ThreadGroup, ChildElement, HttpSampler, LoopController, IfController, TransactionController, ThroughputController, ResponseAssertion, JsonAssertion, DurationAssertion, ConstantTimer, UniformRandomTimer, GaussianRandomTimer, RegexExtractor, JsonExtractor, BoundaryExtractor, UserParameters, KeyValuePair } from '@/types'

/**
 * Parse a JMeter .jmx XML file into an ApiStress TestPlan with proper nested structure.
 *
 * JMeter uses a <hashTree> structure where elements and their children are siblings:
 *   <ThreadGroup .../>
 *   <hashTree>
 *     <HTTPSamplerProxy .../>
 *     <LoopController .../>
 *     <hashTree>...</hashTree>
 *   </hashTree>
 *
 * We walk the XML tree matching elements to their following <hashTree> children.
 */

export function importJmx(xml: string): TestPlan {
  const parser = new DOMParser()
  const doc = parser.parseFromString(xml, 'text/xml')
  const root = doc.documentElement

  // Find the top-level <hashTree> under <jmeterTestPlan>
  let topTree: Element | null = null
  for (const child of root.children) {
    if (child.tagName === 'hashTree') { topTree = child; break }
  }
  if (!topTree) throw new Error('Invalid JMX: no top-level hashTree')

  const children = Array.from(topTree.children)

  // Build TestPlan
  const planId = crypto.randomUUID()
  const plan: TestPlan = {
    id: planId,
    type: 'TestPlan',
    name: 'Imported Test Plan',
    enabled: true,
    comments: '',
    threadGroups: [],
    variables: [],
    listeners: [],
  }

  // Walk top-level elements
  for (let i = 0; i < children.length; i++) {
    const el = children[i]
    if (el.tagName === 'TestPlan') {
      plan.name = getAttr(el, 'testname') || 'Test Plan'
      plan.comments = getAttr(el, 'comments') || ''
      // TestPlan's own hashTree may contain variables
      const nextTree = children[i + 1]
      if (nextTree?.tagName === 'hashTree') {
        parseVariables(nextTree, plan.variables)
      }
    } else if (el.tagName === 'ThreadGroup') {
      const tg = parseThreadGroup(el)
      plan.threadGroups.push(tg)
      // ThreadGroup's children are in the next <hashTree>
      const nextTree = children[i + 1]
      if (nextTree?.tagName === 'hashTree') {
        tg.children = parseChildren(nextTree)
      }
    }
  }

  return plan
}

function parseThreadGroup(el: Element): ThreadGroup {
  return {
    id: crypto.randomUUID(),
    type: 'ThreadGroup',
    name: getAttr(el, 'testname') || 'Thread Group',
    enabled: getAttr(el, 'enabled') !== 'false',
    numThreads: parseInt(getProp(el, 'ThreadGroup.num_threads') || '10'),
    rampUp: parseInt(getProp(el, 'ThreadGroup.ramp_time') || '5'),
    loops: parseInt(getProp(el, 'LoopController.loops') || '1'),
    duration: parseInt(getProp(el, 'ThreadGroup.duration') || '0'),
    delay: parseInt(getProp(el, 'ThreadGroup.delay') || '0'),
    scheduler: getProp(el, 'ThreadGroup.scheduler') === 'true',
    onErrorAction: 'continue',
    sameUserOnEachIteration: true,
    children: [],
  }
}

function parseChildren(hashTree: Element, depth = 0): ChildElement[] {
  if (depth > 20) return [] // safety limit
  const children: ChildElement[] = []
  const elements = Array.from(hashTree.children)

  for (let i = 0; i < elements.length; i++) {
    const el = elements[i]
    const tag = el.tagName
    let child: ChildElement | null = null

    switch (tag) {
      case 'HTTPSamplerProxy':
      case 'HttpSampler':
        child = parseHttpSampler(el)
        break
      case 'LoopController':
        child = parseLoopController(el)
        break
      case 'IfController':
        child = parseIfController(el)
        break
      case 'WhileController':
        child = parseWhileController(el)
        break
      case 'ResponseAssertion':
        child = parseResponseAssertion(el)
        break
      case 'DurationAssertion':
        child = parseDurationAssertion(el)
        break
      case 'ConstantTimer':
        child = parseConstantTimer(el)
        break
      case 'RegexExtractor':
        child = parseRegexExtractor(el)
        break
      case 'JSONPathExtractor':
        child = parseJsonExtractor(el)
        break
      case 'JSONPathAssertion':
        child = parseJsonAssertion(el)
        break
      case 'TransactionController':
        child = parseTransactionController(el)
        break
      case 'ThroughputController':
        child = parseThroughputController(el)
        break
      case 'BoundaryExtractor':
        child = parseBoundaryExtractor(el)
        break
      case 'UniformRandomTimer':
        child = parseUniformRandomTimer(el)
        break
      case 'GaussianRandomTimer':
        child = parseGaussianRandomTimer(el)
        break
      case 'UserParameters':
        child = parseUserParameters(el)
        break
      case 'ResultCollector':
        // Skip — add as listener
        break
      default:
        break
    }

    if (child) {
      children.push(child)
      // Check if the next sibling is a hashTree (children of this element)
      const next = elements[i + 1]
      if (next?.tagName === 'hashTree') {
        i++ // skip the hashTree
        const nested = parseChildren(next, depth + 1)
        if ('children' in child) {
          ;(child as unknown as Record<string, unknown>).children = nested
        }
      }
    }
  }

  return children
}

// ---- Parsers for each element type ----

function parseHttpSampler(el: Element): HttpSampler {
  return {
    id: crypto.randomUUID(),
    type: 'HttpSampler',
    name: getAttr(el, 'testname') || 'HTTP Request',
    enabled: getAttr(el, 'enabled') !== 'false',
    protocol: (getProp(el, 'HTTPSampler.protocol') || 'https') as 'http' | 'https',
    domain: getProp(el, 'HTTPSampler.domain') || '',
    port: parseInt(getProp(el, 'HTTPSampler.port') || '443'),
    path: getProp(el, 'HTTPSampler.path') || '/',
    method: (getProp(el, 'HTTPSampler.method') || 'GET') as HttpSampler['method'],
    headers: parseArgElements(el, 'header'),
    queryParams: parseArgElements(el, 'query'),
    body: parseBody(el),
    auth: { type: 'none' },
    followRedirects: getProp(el, 'HTTPSampler.follow_redirects') !== 'false',
    timeout: parseInt(getProp(el, 'HTTPSampler.connect_timeout') || '30000'),
    useKeepAlive: getProp(el, 'HTTPSampler.use_keepalive') !== 'false',
  }
}

function parseLoopController(el: Element): LoopController {
  return {
    id: crypto.randomUUID(),
    type: 'LoopController',
    name: getAttr(el, 'testname') || 'Loop Controller',
    enabled: getAttr(el, 'enabled') !== 'false',
    loops: parseInt(getProp(el, 'LoopController.loops') || '1'),
    children: [],
  }
}

function parseIfController(el: Element): IfController {
  return {
    id: crypto.randomUUID(),
    type: 'IfController',
    name: getAttr(el, 'testname') || 'If Controller',
    enabled: getAttr(el, 'enabled') !== 'false',
    condition: getProp(el, 'IfController.condition') || '',
    useExpression: getProp(el, 'IfController.useExpression') !== 'false',
    evaluateAll: getProp(el, 'IfController.evaluateAll') === 'true',
    children: [],
  }
}

function parseWhileController(el: Element): ChildElement {
  return {
    id: crypto.randomUUID(),
    type: 'WhileController',
    name: getAttr(el, 'testname') || 'While Controller',
    enabled: getAttr(el, 'enabled') !== 'false',
    condition: getProp(el, 'WhileController.condition') || '',
    children: [],
  } as ChildElement
}

function parseResponseAssertion(el: Element): ResponseAssertion {
  const testFieldMap: Record<string, string> = {
    '2': 'responseCode', '3': 'responseMessage', '16': 'responseBody',
    '5': 'responseHeaders', '4': 'requestHeaders', '1': 'url',
  }
  return {
    id: crypto.randomUUID(),
    type: 'ResponseAssertion',
    name: getAttr(el, 'testname') || 'Response Assertion',
    enabled: getAttr(el, 'enabled') !== 'false',
    testField: (testFieldMap[getProp(el, 'Assertion.test_field') || '2'] || 'responseBody') as ResponseAssertion['testField'],
    patternMatching: mapMatchRule(getProp(el, 'Assertion.test_type') || '2') as ResponseAssertion['patternMatching'],
    patterns: parseStringPropList(el, 'Assertion.ass_test'),
    assumeSuccess: getProp(el, 'Assertion.assume_success') === 'true',
  }
}

function parseDurationAssertion(el: Element): DurationAssertion {
  return {
    id: crypto.randomUUID(),
    type: 'DurationAssertion',
    name: getAttr(el, 'testname') || 'Duration Assertion',
    enabled: getAttr(el, 'enabled') !== 'false',
    maxDuration: parseInt(getProp(el, 'DurationAssertion.duration') || '3000'),
  }
}

function parseConstantTimer(el: Element): ConstantTimer {
  return {
    id: crypto.randomUUID(),
    type: 'ConstantTimer',
    name: getAttr(el, 'testname') || 'Constant Timer',
    enabled: getAttr(el, 'enabled') !== 'false',
    delay: parseInt(getProp(el, 'ConstantTimer.delay') || '300'),
  }
}

function parseRegexExtractor(el: Element): RegexExtractor {
  return {
    id: crypto.randomUUID(),
    type: 'RegexExtractor',
    name: getAttr(el, 'testname') || 'Regex Extractor',
    enabled: getAttr(el, 'enabled') !== 'false',
    referenceName: getProp(el, 'RegexExtractor.refname') || '',
    regex: getProp(el, 'RegexExtractor.regex') || '',
    template: getProp(el, 'RegexExtractor.template') || '$1$',
    matchNo: parseInt(getProp(el, 'RegexExtractor.match_number') || '1'),
    defaultValue: getProp(el, 'RegexExtractor.default') || '',
    useHeaders: false,
    useBody: true,
  }
}

function parseJsonExtractor(el: Element): JsonExtractor {
  return {
    id: crypto.randomUUID(),
    type: 'JsonExtractor',
    name: getAttr(el, 'testname') || 'JSON Extractor',
    enabled: getAttr(el, 'enabled') !== 'false',
    referenceName: getProp(el, 'JSONPathExtractor.var') || '',
    jsonPath: getProp(el, 'JSONPathExtractor.jsonpath') || '$',
    defaultValue: getProp(el, 'JSONPathExtractor.default') || '',
  }
}

// ---- Helpers ----

function getAttr(el: Element, name: string): string | null {
  return el.getAttribute(name)
}

function getProp(el: Element, name: string): string | null {
  for (const prop of el.getElementsByTagName('stringProp')) {
    if (prop.getAttribute('name') === name) return prop.textContent || ''
  }
  for (const prop of el.getElementsByTagName('boolProp')) {
    if (prop.getAttribute('name') === name) return prop.textContent || 'false'
  }
  for (const prop of el.getElementsByTagName('longProp')) {
    if (prop.getAttribute('name') === name) return prop.textContent || '0'
  }
  for (const prop of el.getElementsByTagName('intProp')) {
    if (prop.getAttribute('name') === name) return prop.textContent || '0'
  }
  return null
}

function parseArgElements(el: Element, argType: string): KeyValuePair[] {
  const pairs: KeyValuePair[] = []
  const collection = el.getElementsByTagName('collectionProp')
  for (const col of collection) {
    if (col.getAttribute('name') === `HTTPSampler.${argType}_manager`) {
      for (const ep of col.getElementsByTagName('elementProp')) {
        const name = ep.getAttribute('name') || ''
        const val = parseElementPropString(ep, `${argType === 'header' ? 'Header' : 'Argument'}.value`) || ''
        pairs.push({ key: name, value: val })
      }
    }
  }
  return pairs
}

function parseElementPropString(el: Element, propName: string): string | null {
  for (const sp of el.getElementsByTagName('stringProp')) {
    if (sp.getAttribute('name') === propName) return sp.textContent || ''
  }
  return null
}

function parseStringPropList(el: Element, propName: string): string[] {
  for (const cp of el.getElementsByTagName('collectionProp')) {
    if (cp.getAttribute('name') === propName) {
      return Array.from(cp.getElementsByTagName('stringProp'))
        .map(sp => sp.textContent || '')
        .filter(Boolean)
    }
  }
  return ['']
}

function parseTransactionController(el: Element): TransactionController {
  return {
    id: crypto.randomUUID(),
    type: 'TransactionController',
    name: getAttr(el, 'testname') || 'Transaction Controller',
    enabled: getAttr(el, 'enabled') !== 'false',
    includeDuration: getProp(el, 'TransactionController.includeTimers') === 'true',
    children: [],
  }
}

function parseThroughputController(el: Element): ThroughputController {
  return {
    id: crypto.randomUUID(),
    type: 'ThroughputController',
    name: getAttr(el, 'testname') || 'Throughput Controller',
    enabled: getAttr(el, 'enabled') !== 'false',
    throughput: parseInt(getProp(el, 'ThroughputController.maxThroughput') || '60'),
    perThread: getProp(el, 'ThroughputController.perThread') === 'true',
    children: [],
  }
}

function parseBoundaryExtractor(el: Element): BoundaryExtractor {
  return {
    id: crypto.randomUUID(),
    type: 'BoundaryExtractor',
    name: getAttr(el, 'testname') || 'Boundary Extractor',
    enabled: getAttr(el, 'enabled') !== 'false',
    referenceName: getProp(el, 'BoundaryExtractor.refname') || '',
    leftBoundary: getProp(el, 'BoundaryExtractor.lboundary') || '',
    rightBoundary: getProp(el, 'BoundaryExtractor.rboundary') || '',
    matchNo: parseInt(getProp(el, 'BoundaryExtractor.match_number') || '1'),
    defaultValue: getProp(el, 'BoundaryExtractor.default') || '',
  }
}

function parseJsonAssertion(el: Element): JsonAssertion {
  return {
    id: crypto.randomUUID(),
    type: 'JsonAssertion',
    name: getAttr(el, 'testname') || 'JSON Assertion',
    enabled: getAttr(el, 'enabled') !== 'false',
    jsonPath: getProp(el, 'JSONPathAssertion.jsonpath') || '$',
    expectedValue: getProp(el, 'JSONPathAssertion.expectedValue') || '',
    comparisonMode: 'exists',
    expectNull: getProp(el, 'JSONPathAssertion.expectNull') === 'true',
  }
}

function parseUniformRandomTimer(el: Element): UniformRandomTimer {
  return {
    id: crypto.randomUUID(),
    type: 'UniformRandomTimer',
    name: getAttr(el, 'testname') || 'Uniform Random Timer',
    enabled: getAttr(el, 'enabled') !== 'false',
    minDelay: parseInt(getProp(el, 'UniformRandomTimer.delay') || '0'),
    maxDelay: parseInt(getProp(el, 'UniformRandomTimer.range') || '300'),
  }
}

function parseGaussianRandomTimer(el: Element): GaussianRandomTimer {
  return {
    id: crypto.randomUUID(),
    type: 'GaussianRandomTimer',
    name: getAttr(el, 'testname') || 'Gaussian Random Timer',
    enabled: getAttr(el, 'enabled') !== 'false',
    delay: parseInt(getProp(el, 'GaussianRandomTimer.delay') || '300'),
    deviation: parseInt(getProp(el, 'GaussianRandomTimer.range') || '100'),
  }
}

function parseUserParameters(el: Element): UserParameters {
  const params: KeyValuePair[] = []
  for (const cp of el.getElementsByTagName('collectionProp')) {
    if (cp.getAttribute('name') === 'UserParameters.names') {
      for (const sp of cp.getElementsByTagName('stringProp')) {
        params.push({ key: sp.textContent || '', value: '' })
      }
    }
  }
  return {
    id: crypto.randomUUID(),
    type: 'UserParameters',
    name: getAttr(el, 'testname') || 'User Parameters',
    enabled: getAttr(el, 'enabled') !== 'false',
    parameters: params,
  }
}

function parseBody(el: Element): import('@/types').HttpBody {
  const rawBody = getProp(el, 'HTTPSampler.postBodyRaw')
  if (rawBody) {
    return { mode: 'raw', raw: rawBody, contentType: getProp(el, 'HTTPSampler.content_type') || 'application/json' }
  }
  // Check for multipart/form-data
  const isMultipart = getProp(el, 'HTTPSampler.DO_MULTIPART_POST') === 'true'
  if (isMultipart) {
    const formData: import('@/types').FormDataItem[] = []
    // Collect text parameters
    const args = parseArgElements(el, 'argument')
    for (const arg of args) {
      if (arg.key) formData.push({ key: arg.key, value: arg.value, type: 'text' })
    }
    // Collect file uploads
    for (const cp of el.getElementsByTagName('collectionProp')) {
      if (cp.getAttribute('name') === 'HTTPFileArgs.files') {
        for (const ep of cp.getElementsByTagName('elementProp')) {
          const filePath = parseElementPropString(ep, 'File.path') || ''
          const paramName = parseElementPropString(ep, 'File.paramname') || 'file'
          const mimeType = parseElementPropString(ep, 'File.mimetype') || ''
          if (filePath || paramName) {
            formData.push({
              key: paramName,
              value: filePath,
              type: 'file',
              filename: filePath.split(/[/\\]/).pop() || '',
              mimeType,
            })
          }
        }
      }
    }
    if (formData.length > 0) return { mode: 'form-data', formData }
  }
  const args = parseArgElements(el, 'argument')
  if (args.length > 0) {
    return { mode: 'x-www-form-urlencoded', urlEncoded: args }
  }
  return { mode: 'none' }
}

function parseVariables(hashTree: Element, vars: KeyValuePair[]) {
  for (const el of hashTree.children) {
    if (el.tagName === 'Arguments') {
      for (const cp of el.getElementsByTagName('collectionProp')) {
        for (const ep of cp.getElementsByTagName('elementProp')) {
          const key = ep.getAttribute('name') || ''
          const val = parseElementPropString(ep, 'Argument.value') || ''
          if (key) vars.push({ key, value: val })
        }
      }
    }
  }
}

function mapMatchRule(jmxType: string): string {
  switch (jmxType) {
    case '2': return 'contains'
    case '6': return 'notContains'
    case '1': return 'matches'
    case '8': return 'equals'
    case '16': return 'substring'
    default: return 'contains'
  }
}

// ============================================
// Export: ApiStress TestPlan → JMeter JMX XML
// ============================================

export function exportJmx(plan: TestPlan): string {
  const lines: string[] = []
  lines.push('<?xml version="1.0" encoding="UTF-8"?>')
  lines.push('<jmeterTestPlan version="1.2" properties="5.0" jmeter="5.6.3">')
  lines.push('  <hashTree>')

  // TestPlan element
  pushElement(lines, 2, 'TestPlan', {
    testname: plan.name,
    enabled: plan.enabled ? 'true' : 'false',
    comments: plan.comments || '',
  }, [
    stringProp('TestPlan.comments', plan.comments || ''),
    boolProp('TestPlan.functional_mode', 'false'),
    boolProp('TestPlan.serialize_threadgroups', 'false'),
  ])

  // TestPlan variables
  if (plan.variables.length > 0) {
    pushUserVariables(lines, 2, plan.variables)
  }

  // ThreadGroups with their children
  for (const tg of plan.threadGroups) {
    pushThreadGroup(lines, 2, tg)
  }

  lines.push('  </hashTree>')
  lines.push('</jmeterTestPlan>')
  return lines.join('\n')
}

function pushElement(lines: string[], indent: number, tag: string, attrs: Record<string, string>, props: string[]) {
  const pad = '  '.repeat(indent)
  const attrStr = Object.entries(attrs)
    .map(([k, v]) => `${k}="${escXml(v)}"`)
    .join(' ')
  lines.push(`${pad}<${tag} ${attrStr}>`)
  for (const p of props) lines.push(`${pad}  ${p}`)
  lines.push(`${pad}</${tag}>`)
}

function pushThreadGroup(lines: string[], indent: number, tg: ThreadGroup) {
  const attrs = {
    testname: tg.name,
    enabled: tg.enabled ? 'true' : 'false',
    guiclass: 'ThreadGroupGui',
    testclass: 'ThreadGroup',
  }
  const props = [
    stringProp('ThreadGroup.num_threads', String(tg.numThreads)),
    stringProp('ThreadGroup.ramp_time', String(tg.rampUp)),
    longProp('ThreadGroup.duration', String(tg.duration)),
    longProp('ThreadGroup.delay', String(tg.delay)),
    stringProp('ThreadGroup.on_sample_error', tg.onErrorAction || 'continue'),
    stringProp('LoopController.loops', String(tg.loops)),
    boolProp('ThreadGroup.scheduler', tg.scheduler ? 'true' : 'false'),
  ]
  pushElement(lines, indent, 'ThreadGroup', attrs, props)

  // Children hashTree
  if (tg.children.length > 0) {
    lines.push(`${'  '.repeat(indent)}<hashTree>`)
    for (const child of tg.children) {
      pushChildElement(lines, indent + 1, child)
    }
    lines.push(`${'  '.repeat(indent)}</hashTree>`)
  }
}

function pushChildElement(lines: string[], indent: number, child: ChildElement) {
  const pad = '  '.repeat(indent)
  const kids = 'children' in child ? (child as unknown as Record<string, unknown>).children as ChildElement[] | undefined : undefined

  switch (child.type) {
    case 'HttpSampler': {
      const s = child as unknown as HttpSampler
      lines.push(`${pad}<HTTPSamplerProxy testname="${escXml(s.name)}" enabled="${s.enabled}">`)
      lines.push(`${pad}  <stringProp name="HTTPSampler.domain">${escXml(s.domain)}</stringProp>`)
      lines.push(`${pad}  <stringProp name="HTTPSampler.port">${s.port}</stringProp>`)
      lines.push(`${pad}  <stringProp name="HTTPSampler.protocol">${s.protocol}</stringProp>`)
      lines.push(`${pad}  <stringProp name="HTTPSampler.path">${escXml(s.path)}</stringProp>`)
      lines.push(`${pad}  <stringProp name="HTTPSampler.method">${s.method}</stringProp>`)
      lines.push(`${pad}  <stringProp name="HTTPSampler.connect_timeout">${s.timeout}</stringProp>`)
      lines.push(`${pad}  <boolProp name="HTTPSampler.follow_redirects">${s.followRedirects}</boolProp>`)
      if (s.body.mode === 'raw' && s.body.raw) {
        lines.push(`${pad}  <boolProp name="HTTPSampler.postBodyRaw">true</boolProp>`)
        lines.push(`${pad}  <stringProp name="HTTPSampler.content_type">${escXml(s.body.contentType || 'application/json')}</stringProp>`)
        lines.push(`${pad}  <stringProp name="HTTPSampler.postBody">${escXml(s.body.raw)}</stringProp>`)
      }
      if (s.body.mode === 'x-www-form-urlencoded' && s.body.urlEncoded && s.body.urlEncoded.length > 0) {
        lines.push(`${pad}  <collectionProp name="HTTPSampler.argument_manager">`)
        for (const p of s.body.urlEncoded) {
          if (!p.key) continue
          lines.push(`${pad}    <elementProp name="${escXml(p.key)}" elementType="HTTPArgument">`)
          lines.push(`${pad}      <stringProp name="Argument.name">${escXml(p.key)}</stringProp>`)
          lines.push(`${pad}      <stringProp name="Argument.value">${escXml(p.value)}</stringProp>`)
          lines.push(`${pad}      <boolProp name="HTTPArgument.always_encode">false</boolProp>`)
          lines.push(`${pad}    </elementProp>`)
        }
        lines.push(`${pad}  </collectionProp>`)
      }
      if (s.body.mode === 'form-data' && s.body.formData && s.body.formData.length > 0) {
        lines.push(`${pad}  <boolProp name="HTTPSampler.DO_MULTIPART_POST">true</boolProp>`)
        const textFields = s.body.formData.filter(f => f.type !== 'file')
        const fileFields = s.body.formData.filter(f => f.type === 'file')
        if (textFields.length > 0) {
          lines.push(`${pad}  <collectionProp name="HTTPSampler.argument_manager">`)
          for (const f of textFields) {
            lines.push(`${pad}    <elementProp name="${escXml(f.key)}" elementType="HTTPArgument">`)
            lines.push(`${pad}      <stringProp name="Argument.name">${escXml(f.key)}</stringProp>`)
            lines.push(`${pad}      <stringProp name="Argument.value">${escXml(f.value)}</stringProp>`)
            lines.push(`${pad}      <boolProp name="HTTPArgument.always_encode">false</boolProp>`)
            lines.push(`${pad}    </elementProp>`)
          }
          lines.push(`${pad}  </collectionProp>`)
        }
        if (fileFields.length > 0) {
          lines.push(`${pad}  <collectionProp name="HTTPFileArgs.files">`)
          for (const f of fileFields) {
            const encodedPath = escXml(f.value)
            lines.push(`${pad}    <elementProp name="${escXml(f.filename || f.key)}" elementType="HTTPFileArg">`)
            lines.push(`${pad}      <stringProp name="File.path">${encodedPath}</stringProp>`)
            lines.push(`${pad}      <stringProp name="File.paramname">${escXml(f.key)}</stringProp>`)
            lines.push(`${pad}      <stringProp name="File.mimetype">${escXml(f.mimeType || 'application/octet-stream')}</stringProp>`)
            lines.push(`${pad}    </elementProp>`)
          }
          lines.push(`${pad}  </collectionProp>`)
        }
      }
      if (s.headers.length > 0) {
        lines.push(`${pad}  <collectionProp name="HTTPSampler.header_manager">`)
        for (const h of s.headers) {
          lines.push(`${pad}    <elementProp name="${escXml(h.key)}" elementType="Header">`)
          lines.push(`${pad}      <stringProp name="Header.name">${escXml(h.key)}</stringProp>`)
          lines.push(`${pad}      <stringProp name="Header.value">${escXml(h.value)}</stringProp>`)
          lines.push(`${pad}    </elementProp>`)
        }
        lines.push(`${pad}  </collectionProp>`)
      }
      lines.push(`${pad}</HTTPSamplerProxy>`)
      break
    }
    case 'LoopController': {
      const c = child as unknown as LoopController
      lines.push(`${pad}<LoopController testname="${escXml(c.name)}" enabled="${c.enabled}">`)
      lines.push(`${pad}  <stringProp name="LoopController.loops">${c.loops}</stringProp>`)
      lines.push(`${pad}</LoopController>`)
      break
    }
    case 'IfController': {
      const c = child as unknown as IfController
      lines.push(`${pad}<IfController testname="${escXml(c.name)}" enabled="${c.enabled}">`)
      lines.push(`${pad}  <stringProp name="IfController.condition">${escXml(c.condition)}</stringProp>`)
      lines.push(`${pad}</IfController>`)
      break
    }
    case 'WhileController': {
      const c = child as unknown as { name: string; enabled: boolean; condition: string }
      lines.push(`${pad}<WhileController testname="${escXml(c.name)}" enabled="${c.enabled}">`)
      lines.push(`${pad}  <stringProp name="WhileController.condition">${escXml(c.condition)}</stringProp>`)
      lines.push(`${pad}</WhileController>`)
      break
    }
    case 'ResponseAssertion': {
      const a = child as unknown as ResponseAssertion
      lines.push(`${pad}<ResponseAssertion testname="${escXml(a.name)}" enabled="${a.enabled}">`)
      lines.push(`${pad}  <stringProp name="Assertion.test_field">2</stringProp>`)
      lines.push(`${pad}  <collectionProp name="Assertion.ass_test">`)
      for (const p of a.patterns) {
        lines.push(`${pad}    <stringProp name="0">${escXml(p)}</stringProp>`)
      }
      lines.push(`${pad}  </collectionProp>`)
      lines.push(`${pad}</ResponseAssertion>`)
      break
    }
    case 'DurationAssertion': {
      const a = child as unknown as DurationAssertion
      lines.push(`${pad}<DurationAssertion testname="${escXml(a.name)}" enabled="${a.enabled}">`)
      lines.push(`${pad}  <stringProp name="DurationAssertion.duration">${a.maxDuration}</stringProp>`)
      lines.push(`${pad}</DurationAssertion>`)
      break
    }
    case 'ConstantTimer': {
      const t = child as unknown as ConstantTimer
      lines.push(`${pad}<ConstantTimer testname="${escXml(t.name)}" enabled="${t.enabled}">`)
      lines.push(`${pad}  <stringProp name="ConstantTimer.delay">${t.delay}</stringProp>`)
      lines.push(`${pad}</ConstantTimer>`)
      break
    }
    case 'RegexExtractor': {
      const re = child as unknown as RegexExtractor
      lines.push(`${pad}<RegexExtractor testname="${escXml(re.name)}" enabled="${re.enabled}">`)
      lines.push(`${pad}  <stringProp name="RegexExtractor.refname">${escXml(re.referenceName)}</stringProp>`)
      lines.push(`${pad}  <stringProp name="RegexExtractor.regex">${escXml(re.regex)}</stringProp>`)
      lines.push(`${pad}  <stringProp name="RegexExtractor.template">${escXml(re.template)}</stringProp>`)
      lines.push(`${pad}</RegexExtractor>`)
      break
    }
    case 'TransactionController': {
      const c = child as unknown as TransactionController
      lines.push(`${pad}<TransactionController testname="${escXml(c.name)}" enabled="${c.enabled}">`)
      lines.push(`${pad}  <boolProp name="TransactionController.includeTimers">${c.includeDuration}</boolProp>`)
      lines.push(`${pad}</TransactionController>`)
      break
    }
    case 'ThroughputController': {
      const c = child as unknown as ThroughputController
      lines.push(`${pad}<ThroughputController testname="${escXml(c.name)}" enabled="${c.enabled}">`)
      lines.push(`${pad}  <stringProp name="ThroughputController.maxThroughput">${c.throughput}</stringProp>`)
      lines.push(`${pad}  <boolProp name="ThroughputController.perThread">${c.perThread}</boolProp>`)
      lines.push(`${pad}</ThroughputController>`)
      break
    }
    case 'BoundaryExtractor': {
      const be = child as unknown as BoundaryExtractor
      lines.push(`${pad}<BoundaryExtractor testname="${escXml(be.name)}" enabled="${be.enabled}">`)
      lines.push(`${pad}  <stringProp name="BoundaryExtractor.refname">${escXml(be.referenceName)}</stringProp>`)
      lines.push(`${pad}  <stringProp name="BoundaryExtractor.lboundary">${escXml(be.leftBoundary)}</stringProp>`)
      lines.push(`${pad}  <stringProp name="BoundaryExtractor.rboundary">${escXml(be.rightBoundary)}</stringProp>`)
      lines.push(`${pad}</BoundaryExtractor>`)
      break
    }
    case 'JsonAssertion': {
      const a = child as unknown as JsonAssertion
      lines.push(`${pad}<JSONPathAssertion testname="${escXml(a.name)}" enabled="${a.enabled}">`)
      lines.push(`${pad}  <stringProp name="JSONPathAssertion.jsonpath">${escXml(a.jsonPath)}</stringProp>`)
      lines.push(`${pad}  <stringProp name="JSONPathAssertion.expectedValue">${escXml(a.expectedValue)}</stringProp>`)
      lines.push(`${pad}</JSONPathAssertion>`)
      break
    }
    case 'UniformRandomTimer': {
      const t = child as unknown as UniformRandomTimer
      lines.push(`${pad}<UniformRandomTimer testname="${escXml(t.name)}" enabled="${t.enabled}">`)
      lines.push(`${pad}  <stringProp name="UniformRandomTimer.delay">${t.minDelay}</stringProp>`)
      lines.push(`${pad}  <stringProp name="UniformRandomTimer.range">${t.maxDelay}</stringProp>`)
      lines.push(`${pad}</UniformRandomTimer>`)
      break
    }
    case 'GaussianRandomTimer': {
      const t = child as unknown as GaussianRandomTimer
      lines.push(`${pad}<GaussianRandomTimer testname="${escXml(t.name)}" enabled="${t.enabled}">`)
      lines.push(`${pad}  <stringProp name="GaussianRandomTimer.delay">${t.delay}</stringProp>`)
      lines.push(`${pad}  <stringProp name="GaussianRandomTimer.range">${t.deviation}</stringProp>`)
      lines.push(`${pad}</GaussianRandomTimer>`)
      break
    }
    case 'UserParameters': {
      const up = child as unknown as UserParameters
      lines.push(`${pad}<UserParameters testname="${escXml(up.name)}" enabled="${up.enabled}">`)
      if (up.parameters.length > 0) {
        lines.push(`${pad}  <collectionProp name="UserParameters.names">`)
        for (const p of up.parameters) {
          lines.push(`${pad}    <stringProp name="0">${escXml(p.key)}</stringProp>`)
        }
        lines.push(`${pad}  </collectionProp>`)
      }
      lines.push(`${pad}</UserParameters>`)
      break
    }
    case 'JsonExtractor': {
      const je = child as unknown as JsonExtractor
      lines.push(`${pad}<JSONPathExtractor testname="${escXml(je.name)}" enabled="${je.enabled}">`)
      lines.push(`${pad}  <stringProp name="JSONPathExtractor.var">${escXml(je.referenceName)}</stringProp>`)
      lines.push(`${pad}  <stringProp name="JSONPathExtractor.jsonpath">${escXml(je.jsonPath)}</stringProp>`)
      lines.push(`${pad}  <stringProp name="JSONPathExtractor.default">${escXml(je.defaultValue)}</stringProp>`)
      lines.push(`${pad}</JSONPathExtractor>`)
      break
    }
  }

  // Nested children hashTree
  if (kids && kids.length > 0) {
    lines.push(`${pad}<hashTree>`)
    for (const nested of kids) {
      pushChildElement(lines, indent + 1, nested)
    }
    lines.push(`${pad}</hashTree>`)
  }
}

function pushUserVariables(lines: string[], indent: number, vars: KeyValuePair[]) {
  const pad = '  '.repeat(indent)
  lines.push(`${pad}<Arguments testname="User Defined Variables" enabled="true">`)
  lines.push(`${pad}  <collectionProp name="Arguments.arguments">`)
  for (const v of vars) {
    lines.push(`${pad}    <elementProp name="${escXml(v.key)}" elementType="Argument">`)
    lines.push(`${pad}      <stringProp name="Argument.name">${escXml(v.key)}</stringProp>`)
    lines.push(`${pad}      <stringProp name="Argument.value">${escXml(v.value)}</stringProp>`)
    lines.push(`${pad}    </elementProp>`)
  }
  lines.push(`${pad}  </collectionProp>`)
  lines.push(`${pad}</Arguments>`)
  lines.push(`${pad}<hashTree/>`)
}

// ---- XML helpers ----

function stringProp(name: string, value: string): string {
  return `<stringProp name="${name}">${escXml(value)}</stringProp>`
}

function boolProp(name: string, value: string): string {
  return `<boolProp name="${name}">${value}</boolProp>`
}

function longProp(name: string, value: string): string {
  return `<longProp name="${name}">${value}</longProp>`
}

function escXml(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;')
}
