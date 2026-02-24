
SECTIONS 
{ 
	. = 0x8000;
	.text : {
		*(.text.code16)
		*(.text.code32)
		*(.text.*)
	}

	. = 0x8200;
	.data : {
		*(.data.header)
		*(.data.gdt)
		*(.data.*)
	}

	/DISCARD/ : { 
		*(.note*)
		*(.debug*)
	}
}
