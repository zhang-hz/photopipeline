
import os

BT = chr(96)  # backtick character

def h1(t):
    return f"# {t}"

def h2(t):
    return f"## {t}"

def h3(t):
    return f"### {t}"

def code_block(lang, lines):
    result = [f"{BT*3}{lang}"]
    result.extend(lines)
    result.append(f"{BT*3}")
    return result

def inline_code(s):
    return f"{BT}{s}{BT}"

target = r"../photopipeline/TEST_DESIGN_RUST.md"
# Actually target = r"C:\Data\code\photopipeline\TEST_DESIGN_RUST.md"
target = r"C:\Data\code\photopipeline\TEST_DESIGN_RUST.md"
