import './style.css'
import {Action, Color, Cursor, IndErase, Indication, IndScreen, InitializeIndication, Operation} from "./suite3270.ts";

type GrElement = "underline" | "blink" | "highlight" | "selectable" | "reverse" | "wide" | "order" | "private-use" | "no-copy" | "wrap";
class CharCell {
    td: HTMLElement
    char: string
    fg: Color
    bg: Color
    gr: Set<GrElement>

    constructor(td: HTMLElement) {
        this.td = td;
        this.char = ' ';
        this.fg = "neutralWhite";
        this.bg = "neutralBlack";
        this.gr = new Set<GrElement>();
    }

    update() {
        let cell = this;
        let grset = [];
        for (let grattr of cell.gr) {
            grset.push(grattr);
        }
        cell.td.dataset.gr = grset.join(" ");
        cell.td.dataset.fg = cell.fg;
        cell.td.dataset.bg = cell.bg;
        cell.td.replaceChildren(cell.char)

    }
}

class Js3270 {
    private character_tbl: HTMLTableElement;
    private cgrid: CharCell[][];
    private oia_row: HTMLDivElement;
    private ws: WebSocket;

    private def_fg: Color;
    private def_bg: Color;
    private cursor: Cursor;
    private cursor_cell: CharCell|null;

    constructor(root: HTMLDivElement) {
        // set up default properties
        this.def_fg = "neutralWhite";
        this.def_bg = "neutralBlack";

        this.character_tbl = document.createElement("table");
        this.character_tbl.className = "cgrid";
        this.cursor = {enabled: true, row: 1, column: 1};
        this.cursor_cell = null;
        this.cgrid = [];


        this.oia_row = document.createElement("div");
        // TODO: make this lookup the address based on where it's loaded from
        this.ws = new WebSocket("ws://127.0.0.1:13270/api/ws");
        this.ws.addEventListener("message", this.on_message.bind(this))

        root.replaceChildren(this.character_tbl, this.oia_row);

        window.addEventListener("keydown", this.on_keydown.bind(this))
    }

    private send(actions: Action[]) {
        let op: Operation = {run: {actions: actions}};
        this.ws.send(JSON.stringify(op));
    }

    private keymap: {[k: string]: [string, string[]]} = {
        "PageUp": ["Scroll", ["backward"]],
        "PageDown": ["Scroll", ["forward"]],
        "Backspace": ["Backspace", []],
        "Enter": ["Enter", []],
        "Tab": ["Tab", []],
        "S+Tab": ["Backtab", []],
        "ArrowUp": ["Up", []],
        "ArrowDown": ["Down", []],
        "ArrowRight": ["Right", []],
        "ArrowLeft": ["Left", []],
        "M+r": ["Reset", []],
        "M+a": ["Attn", []],
    }
    private on_keydown(evt: KeyboardEvent) {
        let key_str = [
            evt.ctrlKey? "C+":"",
            evt.altKey?"M+":"",
            evt.shiftKey?"S+":"",
            evt.key
        ].join("");

        if (evt.key.length == 1 && !evt.altKey) {
            this.send([{action: "Key", args: [evt.key]}]);
            evt.stopPropagation();
            return;
        } else if (key_str in this.keymap) {
            let [action, args] = this.keymap[key_str];
            this.send([{action, args}]);
            evt.preventDefault();
            evt.stopPropagation();
        } else if (evt.key[0] == "F" && evt.key.substring(1).match("[0-9]+")) {
            let fn = evt.key.substring(1);
            if (evt.shiftKey) {
                fn = ((fn as any - 0) + 13) + "";
            }
            this.send([{action: "PF", args: [fn]}]);
            evt.stopImmediatePropagation();
            evt.stopPropagation();
            evt.preventDefault();
        } else {
            console.log(key_str);
        }
    }

    private handle_indication(ind: Indication | InitializeIndication) {
        if ("initialize" in ind) {
            for (let subind of ind.initialize) {
                this.handle_indication(subind);
            }
        } else if ("screen-mode" in ind) {
            let mode = ind["screen-mode"];
            this.resize_screen(mode.rows, mode.columns);
        } else if ("screen" in ind) {
            this.handle_ind_screen(ind.screen)
        } else if ("erase" in ind) {
            this.handle_ind_erase(ind.erase);
        }
    }

    private resize_screen(rows: number, columns: number) {
        this.cgrid = [];

        let tbody = document.createElement("tbody");

        for (let y = 0; y < rows; y++) {
            let row: CharCell[] = [];
            let tr = document.createElement("tr");
            for (let x = 0; x < columns; x++) {
                let td = document.createElement("td");
                let cc = new CharCell(td);
                cc.fg = this.def_fg;
                cc.bg = this.def_bg;
                row.push(cc);

                tr.append(td);
            }
            tbody.append(tr);
            this.cgrid.push(row);
        }
        this.redraw_screen();
        this.character_tbl.replaceChildren(tbody);
    }

    private on_message(ev: MessageEvent<any>) {
        let ind = JSON.parse(ev.data) as Indication;
        this.handle_indication(ind);
        // console.log(ind);
    }

    private redraw_screen() {
        for (let row of this.cgrid) {
            for (let cell of row) {
                cell.update()
            }
        }
        this.move_cursor();
    }

    private redraw_region(row: number, col_start: number, count: number) {
        let c_row = this.cgrid[row];
        for (let i = col_start; i < col_start + count; i++) {
            c_row[i].update()
        }
    }

    private move_cursor() {
        if (this.cursor_cell) {
            delete this.cursor_cell.td.dataset.cursor;
            this.cursor_cell = null;
        }
        if (this.cursor.enabled) {
            this.cursor_cell = this.cgrid[this.cursor.row-1][this.cursor.column-1];
            this.cursor_cell.td.dataset.cursor = "";
        }
    }

    private handle_ind_screen(screen: IndScreen) {
        let rows = screen.rows || [];
        for (let row_e of rows) {
            let row_c = this.cgrid[row_e.row - 1];
            for (let change of row_e.changes) {
                let col_start = change.column - 1;
                let count = ("text" in change) ? change.text.length : change.count;
                let text = ("text" in change) ? change.text : null;

                let new_gr: Set<GrElement> | null = null;
                if (change.gr) {
                    new_gr = new Set();
                    new_gr.clear();
                    if (change.gr != "none") {
                        for (let gritem of change.gr.split(",")) {
                            new_gr.add(gritem as GrElement);
                        }
                    }
                }
                for (let i = 0; i < count; i++) {
                    let cell = row_c[col_start+i];

                    cell.fg = change.fg || cell.fg;
                    cell.bg = change.bg || cell.bg;
                    cell.gr = new_gr || cell.gr;
                    if (text) {
                        cell.char = text[i];
                    }
                    cell.update();
                }
            }
        }
        if (screen.cursor) {
            this.cursor = screen.cursor;
            this.move_cursor();
        }
    }


    private handle_ind_erase(erase: IndErase) {
        if ("fg" in erase) {
            this.def_bg = erase.bg
            this.def_fg = erase.fg
        }
        if ("logical_rows" in erase) {
            this.resize_screen(erase.logical_rows, erase.logical_cols);
        } else {
            this.resize_screen(this.cgrid.length, this.cgrid[0].length)
        }
    }
}

let app = document.querySelector<HTMLDivElement>('#app')!;
// @ts-ignore
window.js3270 = new Js3270(app);

