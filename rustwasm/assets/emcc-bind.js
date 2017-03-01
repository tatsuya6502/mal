// Run in node.js context via emscripten.
// Node.js version is v4.1.1 now.
mergeInto(LibraryManager.library, {
    js_tokenizer: function (containerPtr, inputPtr) {
        "use strict";
        let input = Pointer_stringify(inputPtr);

        const append = Module.cwrap("c_jstokenizer_append", void 0, ["number", "string"]);

        const regexp = /[\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"|;.*|[^\s\[\]{}('"`,;)]*)/g;
        while (true) {
            const matches = regexp.exec(input);
            if (!matches) {
                break;
            }
            const match = matches[1];
            if (match === "") {
                break;
            }
            if (match[0] !== ";") {
                append(containerPtr, match);
            }
        }
    }
});
