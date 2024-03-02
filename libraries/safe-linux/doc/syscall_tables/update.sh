#!/bin/bash
# SPDX-License-Identifier: MIT
# Copyright 2023 IROX Contributors
#

curl -O syscall_x86_64.tbl https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/entry/syscalls/syscall_64.tbl
curl -O syscall_i386.tbl https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/x86/entry/syscalls/syscall_32.tbl

