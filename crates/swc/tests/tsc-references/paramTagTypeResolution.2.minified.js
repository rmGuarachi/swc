//// [first.js]
/** @template T
 * @param {T} x
 * @param {(t: T) => void} k
 */ module.exports = function(x, k) {
    return k(x);
};
//// [main.js]
require("./first")(1, function(n) {});
