.section .text
.global __start

__start:
  li $v0, 4045
  syscall
  lui $t0, 0x4000
  subu $v0, $v0, $t0
  sltiu $v0, $v0, 1

# save results
  lui     $s0, 0xbfff         # Load the base address 0xbffffff0
  ori     $s0, 0xfff0
  ori     $s1, $0, 1          # Prepare the 'done' status

  sw      $v0, 8($s0)         # Set the test result
  sw      $s1, 4($s0)         # Set 'done'

$done:
  jr $ra
  nop

