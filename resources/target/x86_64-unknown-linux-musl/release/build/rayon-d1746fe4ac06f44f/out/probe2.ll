; ModuleID = 'probe2.28ecdc97a568b820-cgu.0'
source_filename = "probe2.28ecdc97a568b820-cgu.0"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-musl"

; probe2::probe
; Function Attrs: nonlazybind uwtable
define void @_ZN6probe25probe17h837dbfe6e9dba8ceE() unnamed_addr #0 {
start:
  ret void
}

attributes #0 = { nonlazybind uwtable "probe-stack"="inline-asm" "target-cpu"="x86-64" }

!llvm.module.flags = !{!0, !1}
!llvm.ident = !{!2}

!0 = !{i32 8, !"PIC Level", i32 2}
!1 = !{i32 2, !"RtLibUseGOT", i32 1}
!2 = !{!"rustc version 1.84.1 (e71f9a9a9 2025-01-27)"}
