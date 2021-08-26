" Theme: VivalloColors
" Author: Andres Vivallo <andresvivallo4@gmail.com>
" License: MIT
"

let s:version = '0.0.1'

" guifg, guibg: The colors of your graphical vim (gui = graphical user interface) 
" guifg is for your text color, and guibg is for the background.
" gui: The text style. Can be bold, italic or underline (as far as I know).
" ctermfg, ctermbg, cterm: same as guifg, guibg and gui, but for the terminal.


" Clear any highlights that were defined before
highlight clear
if exists("syntax_on")
	syntax reset
endif

set background=dark
