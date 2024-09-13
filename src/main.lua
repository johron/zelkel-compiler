local stack = {}

function stack:push(value)
    table.insert(stack, value)
end

function stack:pop()
    return table.remove(stack)
end

local function push(number)
    stack:push(number)
end

local function add()
    local a = stack:pop()
    local b = stack:pop()

    table.insert(stack, b + a)
end

local function sub()
    local a = stack:pop()
    local b = stack:pop()

    table.insert(stack, b - a)
end

local function mul()
    local a = stack:pop()
    local b = stack:pop()

    table.insert(stack, b * a)
end

local function div()
    local a = stack:pop()
    local b = stack:pop()

    table.insert(stack, b / a)
end

local function rot()
    local a = stack:pop()
    local b = stack:pop()

    table.insert(stack, a)
    table.insert(stack, b)
end

local function dup()
    local a = stack:pop()

    table.insert(stack, a)
    table.insert(stack, a)
end

local function echo()
    print(stack:pop())
end

local function peek()
    local a = stack:pop()
    stack:push(a)
    print(a)
end

--[[local program = {
    push(5),
    push(5),
    add(),
    push(3),
    mul(),
    peek(),
    push(2),
    rot(),
    div(),
    push(5),
    push(2),
    sub(),
    rot(),
    sub(),
    echo()
}]]

local function print_table(table)
    for i in pairs(table) do
        print(table[i])
    end
end

local function is_alpha(str)
    return str:match("^[a-zA-Z_]+$") ~= nil
end

local function check_pattern(str, pat)
    local value = string.match(str, pat)

    if value then
        return value
    else
        return nil
    end
end

local function has_value(arr, val)
    for index, value in ipairs(arr) do
        if value == val then
            return true
        end
    end

    return false
end

local file = io.open("test.stabel", "rb")
if not file then print("No 'test.stabel' file") os.exit(1)  end
local content = file:read("a")
file:close()

print(content)
print("div\n")


--[[
 - TYPE(val?, ...)
 -
 - INT(val) > integer
 - ID(val) > identifier
 - OP(+|-|*|/)
 - ]]


local chars = {}
for i = 1, #content do
    table.insert(chars, content:sub(i, i))
end

local toks = {}
local i = 1
while i <= #chars do
    local c = chars[i]
    if tonumber(c) then
        local num = c
        local j = i + 1
        while chars[j] and tonumber(chars[j]) do
            num = num .. chars[j]
            j = j + 1
        end
        table.insert(toks, "INT(" .. num .. ")")
        i = j - 1
    elseif is_alpha(c) then
        local str = c
        local j = i + 1
        while chars[j] and is_alpha(chars[j]) do
            str = str .. chars[j]
            j = j + 1
        end
        table.insert(toks, "ID(" .. str .. ")")
        i = j - 1
    elseif c == "+" or c == "-" or c == "*" or c == "/" or c == "@" or c == ":" or c == "=" or c == "!" then
        table.insert(toks, "OP(" .. c .. ")")
    elseif c == " " or c == "\n" then -- disregard
    else
        print("Unrecognized character: '" .. c .. "'")
        os.exit(1)
    end

    i = i + 1
end

print_table(toks)

print("div\n")

local p = {
    int = "^INT%((%-?%d+)%)$",
    op = "^OP%(([%+%-%*%@%:%=%!/])%)$",
    id = "^ID%(([a-zA-Z_]+)%)$"
}

local code = [[
#include <stdio.h>
#include <stdlib.h>

#define MAX_SIZE 255

int stack[MAX_SIZE];
int top = -1;
int __a__;
int __b__;

void push(int item) {
if (top == MAX_SIZE - 1) {
printf("Stack Overflow\n");
exit(1);
}
stack[++top] = item;
}

int pop() {
if (top == -1) {
printf("Stack Underflow\n");
exit(1);
}
return stack[top--];
}

void main() {
]]

local defs = {}

i = 1
while i <= #toks do
    local t = toks[i]
    code = code .. "// " .. t .. "\n"
    if check_pattern(t, p.int) then
        local value = check_pattern(t, p.int)
        code = code .. "push(" .. value .. ");\n"
    elseif check_pattern(t, p.id) then
        local value = check_pattern(t, p.id)
        if value == "echo" then
            code = code .. "printf(\"%d\\n\", pop());\n"
        elseif value == "peek" then
            code = code .. "__a__ = pop();\n"
            code = code .. "push(__a__);\n"
            code = code .. "printf(\"%d\\n\", __a__);\n"
        elseif value == "end" then
            code = code .. "}\n"
        elseif value == "then" then
            if not (toks[i - 1] and (check_pattern(toks[i - 1], p.op) == "=" or check_pattern(toks[i - 1], p.op) == "!")) then
                print("The word `then` must have an equal or not equal symbol")
                os.exit(1)
            end
        elseif value == "def" then
            local var = check_pattern(toks[i - 1], p.id)
            if toks[i - 1] and var then
                if has_value(defs, var) then
                    code = code .. var .. " = pop();\n"
                else
                    code = code .. "int " .. var .. " = pop();\n"
                    table.insert(defs, var)
                end
            else
                print("The word `def` must have an identifier before it")
                os.exit(1)
            end
        elseif has_value(defs, value) then
            if toks[i + 1] and check_pattern(toks[i + 1], p.id) ~= "def" then
                code = code .. "push(" .. value .. ");\n"
            end
        else
            if not (toks[i + 1] and check_pattern(toks[i + 1], p.id) == "def") then
                print("Unrecognized identifier: " .. value)
                os.exit(1)
            end
        end
    elseif check_pattern(t, p.op) then
        local s = check_pattern(t, p.op)
        if s == "+" then
            code = code .. "__a__ = pop();\n"
            code = code .. "push(pop() + __a__);\n"
        elseif s == "-" then
            code = code .. "__a__ = pop();\n"
            code = code .. "push(pop() - __a__);\n"
        elseif s == "*" then
            code = code .. "__a__ = pop();\n"
            code = code .. "push(pop() * __a__);\n"
        elseif s == "/" then
            code = code .. "__a__ = pop();\n"
            code = code .. "push(pop() / __a__);\n"
        elseif s == "@" then
            code = code .. "__a__ = pop();\n"
            code = code .. "__b__ = pop();\n"
            code = code .. "push(__a__);\n"
            code = code .. "push(__b__);\n"
        elseif s == ":" then
            code = code .. "__a__ = pop();\n"
            code = code .. "push(__a__);\n"
            code = code .. "push(__a__);\n"
        elseif s == "=" then
            code = code .. "__b__ = pop();\n"
            code = code .. "__a__ = pop();\n"
            if toks[i + 1] and check_pattern(toks[i + 1], p.id) == "then" then
                code = code .. "if (__a__ == __b__) {\n"
            else
                code = code .. "push(__a__ == __b__);\n"
            end
        elseif s == "!" then
            code = code .. "} else {\n"
        else
            print("Unrecognized token " .. t)
            os.exit(1)
        end
    end

    if i < #toks then
        code = code .. "\n"
    end

    i = i + 1
end

code = code .. "}"
local out = io.open("test.c", "w")
if out == nil then
    print("file is nil")
    os.exit(1)
end
out:write(code)
out:close()

os.execute("gcc test.c -o test.out")