REC   START 0


main

.uporaba:
. PUSH
.STA   @stackptr    . na vrh sklada zapišemo A
.JSUB   stackpush   . povečamo kazalec na vrh sklada

. POP
.JSUB   stackpop    . zmanjšamo kazalec na vrh sklada
.LDA   @stackptr    . z vrha sklada preberemo A


	. stack init
	LDA #stack
	STA stackptr
	LDA #0

	. klic fakultete (pričakuje argument v A registru, vanj zapiše tudi rezultat)
rid	RD input
	COMP #10
	JEQ kn_sub
	J nprj

kn_sub  JSUB konc_stv
	LDA #0
	STA stevilo
	J rid

nprj	SUB #48
	COMP #0
	JEQ halt

	RMO A, T
	LDA stevilo
	MUL #10
	ADDR T, A
	STA stevilo
	J rid


halt 	J halt
	END main


. pricakuje stevilo v "stevilo" naslovu
konc_stv
	STL @stackptr
	JSUB stackpush

	LDA stevilo
	JSUB fax

	JSUB izpis
	LDA #10
	WD stdout

	JSUB stackpop
	LDL @stackptr

	RSUB


izpis
	STL @stackptr
	JSUB stackpush

	STA stevilo
	LDX #1
to10	
	JSUB mod
	ADD #48

	STA @stackptr
	JSUB stackpush
	
	LDA stevilo
	DIV #10
	STA stevilo

	COMP #0
	JEQ prt
	TIX stevilo
	J to10

prt
	JSUB stackpop
	LDA @stackptr

	WD stdout
	RMO X, A
	SUB #1
	RMO A, X
	COMP #0
	JGT prt

	JSUB stackpop
	LDL @stackptr
	RSUB

mod
	RMO A, T
	DIV #10
	MUL #10
	SUBR A, T
	RMO T, A
	RSUB

. ------------------------------------------------------------------------
. izracuna fakulteto stevila, ki je podan v registru A, rezultat vrne v A
. fn fax(st: u16) -> u16 {  // Ubistvu u24 :P
.	if st == 0 {
. 	   return 1;
.	}
.	// Na dolgo in široko, da si lažje preslikam v assembly
.	let nxt = st - 1;
.	let nxt = fax(nxt);
.	let nxt = st * nxt;
.
.	return nxt;
. }
.
fax      
	. test če je število == 0 ...
	COMP #0
	JGT noz

. st == 0
	. return 1
	LDA #1
	RSUB


. st >= 1
noz
	. push registrov na sklad
	STL @stackptr
	JSUB stackpush
	
	STA @stackptr
	JSUB stackpush
	. stack = [L, A]

	SUB #1
	. stack = [L, A], register A = A - 1 
	JSUB fax
	. stack = [L, A], register A = rezultat izracuna
	JSUB stackpop  . pop A
	MUL @stackptr
	. stack = [L]

	JSUB stackpop
	. stack = []
	LDL @stackptr
	RSUB
. ---------------------------------------------------------------------------


. --------------------------------
. stack functions
stackpush
	STA olda
	LDA stackptr
	ADD #3
	STA stackptr
	LDA olda
	RSUB

stackpop
	STA olda
	LDA stackptr
	SUB #3
	STA stackptr
	LDA olda
	RSUB

. stack data
stack		RESW 1000
stackptr	WORD 0
olda 		RESW 1	
. ---------------------------------

input		WORD X'FA'
stdout 		WORD X'A1'
stevilo		WORD 0
