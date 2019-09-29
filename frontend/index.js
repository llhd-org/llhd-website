var editor_hdl = ace.edit("editor-hdl");
var editor_llhd = ace.edit("editor-llhd");

editor_hdl.setOptions({
    showPrintMargin: false,
    theme: "ace/theme/tomorrow",
    mode: "ace/mode/verilog",
});

editor_llhd.setOptions({
    showPrintMargin: false,
    theme: "ace/theme/tomorrow",
    mode: "ace/mode/text",
    readOnly: true,
});

// Keep track of the code across page reloads.
if (!localStorage.code) {
    localStorage.code = "module Accumulator (\n\
    input  logic        clk,\n\
    input  logic        direction,\n\
    input  logic [15:0] increment,\n\
    output logic [15:0] result\n\
);\n\
    logic [15:0] next;\n\
\n\
    always_comb begin\n\
        if (direction)\n\
            next = result + increment;\n\
        else\n\
            next = result - increment;\n\
    end\n\
\n\
    always_ff @(posedge clk)\n\
        result <= next;\n\
endmodule";
}
editor_hdl.session.setValue(localStorage.code);
editor_hdl.session.on("change", function () {
    localStorage.code = editor_hdl.session.getValue();
});
