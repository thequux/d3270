export type CodePage = {
    name: string,
    aliases?: string[],
}

export type ActionCause =
    "command" | "default" | "file-transfer" | "httpd" | "idle" |
    "keymap" | "macro" | "none" | "password" | "paste" | "peek" |
    "screen-redraw" | "script" | "typeahead" | "ui"

export type ConnectionState =
    "not-connected" | "reconnecting" | "resolving" |
    "tcp-pending" | "tls-pending" | "telnet-pending" |
    "connected-nvt" | "connected-nvt-charmode" |
    "connected-3270" | "connected-unbound" |
    "connected-e-nvt" | "connected-sscp" |
    "connected-tn3270e"

export type IndConnection = {
    state: ConnectionState,
    host?: string,
    cause?: ActionCause,
}

export type ComposeType = "std" | "ge"
export type Color =
    "neutralBlack" | "blue" | "red" | "pink" | "green" | "turquoise" |
    "yellow" | "neutralWhite" | "black" | "deepBlue" | "orange" | "purple" |
    "paleGreen" | "paleTurquoise" | "gray" | "white"

export type IndErase = ({logical_rows: number, logical_cols: number} | {}) &
    ({fg: Color, bg: Color} | {})

export type IndHello = {version: string, build: string, copyright: string}
export type IndModel = {
    model: number,
    rows: number,
    columns: number,
}

export type OiaCompose = { field: "compose"} &
    (
        {value: true, char: string, type: ComposeType} |
        {value: false}
    )
export type OiaInsert = {field: "insert", value: boolean}
export type OiaLock = { field: "lock", value?: string }
export type OiaLu = { field: "lu", value: string, lu?: string }
export type OiaNotUndera = { field: "not-undera", value: boolean }
export type OiaReverseInput = { field: "reverse-input", value: boolean }
export type OiaScreenTrace = { field: "screen-trace", value?: number }
export type OiaScript = { field: "script", value: boolean }
export type OiaTiming = { field: "timing", value?: string }
export type OiaTypeahead = { field: "typeahead", value: boolean }
export type IndOia =
    OiaCompose | OiaInsert | OiaLock | OiaLu | OiaNotUndera | OiaReverseInput |
    OiaScreenTrace | OiaScript | OiaTiming | OiaTypeahead
export type OiaFieldName = IndOia["field"]

export type IndProxy = {
    name: string,
    username: boolean,
    port?: number,
}

export type IndSetting = {
    name: string,
    value?: any, // Todo: enhance this
    cause?: ActionCause,
}

export type IndScreenMode = {
    model: number,
    rows: number,
    columns: number,
    color: boolean,
    oversize: boolean,
    extended: boolean,
}

export type IndTlsHello = {
    supported: boolean,
    provider: string,
    options?: string[],
}

export type IndTls =
    ( { secure: true, verified: boolean } | { secure:false } ) &
    {
        session?: string,
        "host-cert"?: string,
    }

export type IndConnectAttempt = { "host-ip": string, port: string }
export type Cursor = {enabled: false} | {enabled:true, row: number, column: number }
export type IndFileTransfer = { cause: ActionCause } & (
    { state: "awaiting" } |
    { state: "running", bytes: number } |
    { state: "aborting" } |
    { state: "complete", text: string, success: boolean }
    )

export type IndPassthru = {
    "p-tag": string,
    "parent-r-tag"?: string,
    action: string,
    args?: string[],
}

export type IndPopup = {
    type: "connect-error" | "error" | "info" | "result" | "printer" | "child"
    text: string,
    // todo: refine
    error?: string,
}

export type Change = {
    column: number
    // TODO: determine if this is a
    gr?: string, // there's more structure than this, but can't represent in TS
    fg?: Color,
    bg?: Color,
} & ({count: number} | {text: string})

export type IndScreen = {
    cursor?: Cursor,
    rows?: {
        row: number,
        changes: Change[]
    }[]
}

export type IndRunResult = {
    "r-tag"?: string,
    success: boolean,
    text?: string[]
    abort?: boolean
    time: number
}

export type IndScroll = {
    // TODO: refine
    fg?: Color,
    bg?: Color,
}

export type IndStats = {
    "bytes-received": number,
    "bytes-sent": number,
    "records-received": number,
    "records-sent": number,
}

export type IndTerminalName = {
    text: string,
    override: boolean,
}

export type IndThumb = {
    top: number,
    shown: number
    saved: number
    screen: number
    back: number
}

export type IndTraceFile = {
    name?: string
}

export type IndUiError = {
    fatal: boolean
    text: string
    operation?: string
    member?: string
    line?: number
    column?: number
}

// operations
export type OpRun = {
    "r-tag"?: string,
    type?: string,
    // This is true of d3270, but b3270 offers more options
    actions: Action[]
}

export type Action = {
    action: string,
    args?: string[],
}

export type OpRegister = {
    name: string,
    "help-text"?: string,
    "help-params": string,
}

export type OpResult = {
    "p-tag": string,
    text?: string[]
}

export type Operation =
    {run: OpRun} |
    {register: OpRegister} |
    {fail: OpResult} |
    {succeed: OpResult}

export type Indication =
    {bell: {}} |
    {connection: IndConnection} |
    {"connect-attempt": IndConnectAttempt} |
    {erase: IndErase} |
    {flipped: {value: boolean}} |
    {font: {text: string}} |
    {formatted: {state: boolean}} |
    {ft: IndFileTransfer} |
    {icon: {text: string}} |
    {initialize: InitializeIndication[]} |
    {oia: IndOia} |
    {passthru: IndPassthru} |
    {popup: IndPopup} |
    {"run-result": IndRunResult} |
    {screen: IndScreen} |
    {"screen-mode": IndScreenMode} |
    {scroll: IndScroll} |
    {setting: IndSetting} |
    {stats: IndStats} |
    {thumb: IndThumb} |
    {"trace-file": IndTraceFile} |
    {tls: IndTls} |
    {"ui-error": IndUiError} |
    {"window-title": {text: string}}

export type InitializeIndication =
    {"code-pages": CodePage[]} |
    {connection: IndConnection} |
    {erase: IndErase} |
    {hello: IndHello} |
    {models: IndModel} |
    {oia: IndOia} |
    {prefixes: {value: string}} |
    {proxies: IndProxy[]} |
    {"screen-mode": IndScreenMode} |
    {setting: IndSetting} |
    {"terminal-name": IndTerminalName} |
    {thumb: IndThumb} |
    {"tls-hello": IndTlsHello} |
    {tls: IndTls} |
    {"trace-file": IndTraceFile}
