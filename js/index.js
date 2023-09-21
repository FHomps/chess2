let pieces = {
    "standard": `\
rnbqkbnr
pppppppp
________
________
________
________
PPPPPPPP
RNBQKBNR`,
    "hole": `\
rnbqkbnr
pppppppp
________
___XX___
___XX___
________
PPPPPPPP
RNBQKBNR`,
    "closeup": `\
rnbqkbnr
pppppppp
________
________
________
PPPPPPPP
RNBQKBNR`,
    "marathon": `\
rnbqkbnr
pppppppp
________
________
________
________
________
PPPPPPPP
RNBQKBNR`,
    "custom": ""
}

let promotions = {
    "standard": `\
WWWWWWWW
________
________
________
________
________
________
bbbbbbbb`,
    "hole": `\
WWWWWWWW
________
________
___XX___
___XX___
________
________
bbbbbbbb`,
    "closeup": `\
WWWWWWWW
________
________
________
________
________
bbbbbbbb`,
    "marathon": `\
WWWWWWWW
________
________
________
________
________
________
________
bbbbbbbb`,
    "custom": ""
}

let layout_select = document.getElementById("layout_select")
let pieces_ta = document.getElementById("pieces_ta")
let promotions_ta = document.getElementById("promotions_ta")

layout_select.onchange = function() {
    let selected = layout_select.value
    pieces_ta.value = pieces[selected]
    promotions_ta.value = promotions[selected]
}
layout_select.onchange()

pieces_ta.onchange = function() {
    pieces[layout_select.value] = pieces_ta.value
}

promotions_ta.onchange = function() {
    promotions[layout_select.value] = promotions_ta.value
}