//// [requires.d.ts]
//// [mod1.js]
/// <reference path='./requires.d.ts' />
function A() {
    this.p = 1;
}
module.exports.bothBefore = "string", A.justExport = 4, A.bothBefore = 2, A.bothAfter = 3, module.exports = A, module.exports.bothAfter = "string", module.exports.justProperty = "string";
//// [a.js]
/// <reference path='./requires.d.ts' />
var mod1 = require("./mod1");
mod1.justExport.toFixed(), mod1.bothBefore.toFixed() // error
, mod1.bothAfter.toFixed(), mod1.justProperty.length;
