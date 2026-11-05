# Decisions log

Times are written as UTC (MSK).

| When | Decision | Why |
|---|---|---|
| Nov 5 | OutLayer is the primary TEE; NEAR AI Cloud is used only for the optional trade explainer | OutLayer runs arbitrary WASM, has sealed secrets, and calls back into NEAR contracts; NEAR AI Cloud attests inference only |
| Nov 5 | 1Click is `dry: true` everywhere until the jurisdiction question is answered | 1Click ToS restricted-jurisdiction clause, no published list |
| Nov 8 19:00 UTC (22:00 MSK) | GATE-1: _TBD_ | |
