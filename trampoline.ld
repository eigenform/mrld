
SECTIONS 
{ 
	. = 0x8000;
	.text : { 
		*(.text.code16)
		*(.text.code32)
		*(.text.*)
	}

	. = ALIGN(64);
	.data : { 
		*(.data.*)
	}

	/DISCARD/ : { 
		*(.note*)
		*(.debug*)
	}
}
