"""
tvcc-ctx.py - GDB context display for tvcc (Rosetta remote debug compatible)

Commands:
  ctx        - Show registers + stack + disassembly
  stack [n]  - Show stack (default 16 entries)
  ctx-on     - Enable auto-display on stop
  ctx-off    - Disable auto-display on stop
"""

import gdb

# ANSI color codes
RED = "\033[91m"
GREEN = "\033[92m"
YELLOW = "\033[93m"
CYAN = "\033[96m"
BOLD = "\033[1m"
DIM = "\033[2m"
RESET = "\033[0m"

SEPARATOR = DIM + "-" * 60 + RESET

REGS = ["rax", "rdi", "rsi", "rdx", "rcx", "rbx", "rbp", "rsp", "rip"]

# Layout: pairs of registers per line
REG_PAIRS = [
    ("rax", "rdi"),
    ("rcx", "rsi"),
    ("rdx", "rbx"),
    ("rbp", "rsp"),
    ("rip",),
]

# State
_prev_regs = {}
_auto_display = True


def _read_reg(name):
    """Read a register value. Returns int or None on failure."""
    try:
        val = gdb.parse_and_eval("$" + name)
        return int(val) & 0xFFFFFFFFFFFFFFFF
    except Exception:
        return None


def _read_all_regs():
    """Read all tracked registers. Returns dict name->int."""
    regs = {}
    for r in REGS:
        v = _read_reg(r)
        if v is not None:
            regs[r] = v
    return regs


def _fmt_reg(name, val, changed):
    """Format a single register: name : 0x... with optional color."""
    color = RED + BOLD if changed else ""
    reset = RESET if changed else ""
    return f"  {name:<4}: {color}0x{val:016x}{reset}"


def _show_registers():
    """Display registers in 2-column layout with change highlighting."""
    global _prev_regs

    regs = _read_all_regs()
    if not regs:
        print("  <cannot read registers>")
        _prev_regs = {}
        return

    print(SEPARATOR)
    print(BOLD + "[ Registers ]" + RESET)

    for pair in REG_PAIRS:
        parts = []
        for name in pair:
            val = regs.get(name)
            if val is None:
                parts.append(f"  {name:<4}: <unavailable>")
            else:
                changed = name in _prev_regs and _prev_regs[name] != val
                parts.append(_fmt_reg(name, val, changed))
        print(" | ".join(parts))

    _prev_regs = regs


def _show_stack(count=16):
    """Display stack entries from RSP."""
    rsp = _read_reg("rsp")
    rbp = _read_reg("rbp")

    print(SEPARATOR)
    print(BOLD + "[ Stack (from RSP) ]" + RESET)

    if rsp is None:
        print("  <cannot read RSP>")
        return

    try:
        output = gdb.execute(f"x/{count}gx $rsp", to_string=True)
    except Exception:
        print("  <cannot read stack memory>")
        return

    for line in output.strip().split("\n"):
        # GDB output format: "0x7fff...: 0x00000..."
        # Each line may have 1 or 2 values
        parts = line.split(":\t")
        if len(parts) < 2:
            parts = line.split(":\t")
            if len(parts) < 2:
                print("  " + line.strip())
                continue

        addr_str = parts[0].strip()
        values_str = parts[1].strip()

        try:
            addr = int(addr_str, 16)
        except ValueError:
            print("  " + line.strip())
            continue

        # GDB may pack multiple values per line
        val_strs = values_str.split("\t")
        for i, vs in enumerate(val_strs):
            vs = vs.strip()
            if not vs:
                continue

            entry_addr = addr + i * 8
            try:
                val = int(vs, 16)
            except ValueError:
                val = None

            marker = ""
            if rsp is not None and entry_addr == rsp:
                marker = CYAN + "  <-- RSP" + RESET
            if rbp is not None and entry_addr == rbp:
                marker = GREEN + "  <-- RBP" + RESET

            decimal = ""
            if val is not None and 0 < val < 0x10000:
                decimal = DIM + f"  ({val})" + RESET

            if val is not None:
                print(f"  0x{entry_addr:x} : 0x{val:016x}{decimal}{marker}")
            else:
                print(f"  0x{entry_addr:x} : {vs}{marker}")


def _show_disasm():
    """Display disassembly around current PC."""
    print(SEPARATOR)
    print(BOLD + "[ Disassembly ]" + RESET)

    output = None
    # Try disassemble first (shows full function)
    try:
        output = gdb.execute("disassemble", to_string=True)
    except Exception:
        pass

    # Fallback to x/Ni
    if output is None or "Cannot" in output:
        try:
            output = gdb.execute("x/11i $pc-16", to_string=True)
        except Exception:
            try:
                output = gdb.execute("x/7i $pc", to_string=True)
            except Exception:
                print("  <cannot disassemble>")
                return

    if output is None:
        print("  <cannot disassemble>")
        return

    rip = _read_reg("rip")

    for line in output.strip().split("\n"):
        stripped = line.strip()
        if not stripped:
            continue

        # Skip "Dump of assembler" / "End of assembler" lines
        if stripped.startswith("Dump of") or stripped.startswith("End of"):
            continue

        is_current = "=>" in stripped
        # Also check by address match
        if not is_current and rip is not None:
            try:
                # Extract address from line like "0x401020 <main>: push rbp"
                addr_part = stripped.split()[0].lstrip("=>").strip()
                if addr_part.startswith("0x"):
                    line_addr = int(addr_part, 16)
                    is_current = line_addr == rip
            except (ValueError, IndexError):
                pass

        if is_current:
            print(YELLOW + BOLD + "   " + stripped + RESET)
        else:
            print("   " + stripped)

    print(SEPARATOR)


def show_context(count=16):
    """Display full context: registers + stack + disassembly."""
    _show_registers()
    _show_stack(count)
    _show_disasm()


# ---- Stop event handler ----

def _stop_handler(event):
    if _auto_display:
        show_context()


# ---- GDB Commands ----

class CtxCommand(gdb.Command):
    """Show registers, stack, and disassembly."""

    def __init__(self):
        super().__init__("ctx", gdb.COMMAND_USER)

    def invoke(self, arg, from_tty):
        show_context()


class StackCommand(gdb.Command):
    """Show stack entries. Usage: stack [count]"""

    def __init__(self):
        super().__init__("stack", gdb.COMMAND_USER)

    def invoke(self, arg, from_tty):
        count = 16
        if arg.strip():
            try:
                count = int(arg.strip())
            except ValueError:
                print("Usage: stack [count]")
                return
        _show_stack(count)


class CtxOnCommand(gdb.Command):
    """Enable automatic context display on stop."""

    def __init__(self):
        super().__init__("ctx-on", gdb.COMMAND_USER)

    def invoke(self, arg, from_tty):
        global _auto_display
        _auto_display = True
        print("[tvcc-ctx] Auto-display enabled.")


class CtxOffCommand(gdb.Command):
    """Disable automatic context display on stop."""

    def __init__(self):
        super().__init__("ctx-off", gdb.COMMAND_USER)

    def invoke(self, arg, from_tty):
        global _auto_display
        _auto_display = False
        print("[tvcc-ctx] Auto-display disabled.")


# ---- Initialize ----

CtxCommand()
StackCommand()
CtxOnCommand()
CtxOffCommand()

gdb.events.stop.connect(_stop_handler)

print("[tvcc-ctx] Context display loaded. Commands: ctx, stack [n], ctx-on, ctx-off")
