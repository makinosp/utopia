root := justfile_directory()
tools_path := root + "/.aidlc/tools"

[group('aidlc')]
doctor:
    bun {{tools_path}}/aidlc-utility.ts doctor

[group('aidlc')]
status:
    bun {{tools_path}}/aidlc-utility.ts status

[group('aidlc')]
scopes:
    bun {{tools_path}}/aidlc-utility.ts scope-table

[group('aidlc')]
compile:
    bun {{tools_path}}/aidlc-graph.ts compile

[group('aidlc')]
validate scope:
    bun {{tools_path}}/aidlc-graph.ts validate-scope {{scope}}
