
/* Base physical address where the kernel is loaded */
_kernel_phys_base   = 0x04000000;

/* Base virtual address where the kernel resides during runtime */
_kernel_virt_base   = 0xffffffff80000000;

/* 2MiB stack size, so we can use a single 2MiB page */
_kernel_stack_size  = 0x00200000;

/* 2MiB interrupt stack size, so we can use a single 2MiB page */
_kernel_istack_size  = 0x00200000;


ENTRY(_start);

PHDRS {
	text PT_LOAD;
	data PT_LOAD;
}

SECTIONS
{
	. = _kernel_virt_base;
	.start ALIGN(2M) : AT(_kernel_phys_base)
	{
		*(.start)
		*(.start.*)
	} :text

	.text :
	{
		*(.text)
		*(.text.*)
	} :text

	.got ALIGN(2M) :
	{
		*(.got)
	} :data

	.rodata :
	{
		*(.rodata)
		*(.rodata.*)
	} :data

	.data :
	{
		*(.data)
		*(.data.*)
	} :data

	.bss :
	{
		*(.bss)
		*(.bss.*)
	} :data

	.stack ALIGN(2M) (NOLOAD) :
	{
		_kernel_stack_lo = .;
		. = _kernel_stack_lo + _kernel_stack_size;
		_kernel_stack_hi = .;
	} :data

	.istack ALIGN(2M) (NOLOAD) :
	{
		_kernel_istack_lo = .;
		. = _kernel_istack_lo + _kernel_istack_size;
		_kernel_istack_hi = .;
	} :data

	_kernel_heap_base = .;

	/DISCARD/ : {
		*(.comment*)
		*(.note*)
		*(.eh_frame*)
	}

}
