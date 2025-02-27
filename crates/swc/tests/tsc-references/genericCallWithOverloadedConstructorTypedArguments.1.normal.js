//// [genericCallWithOverloadedConstructorTypedArguments.ts]
// Function typed arguments with multiple signatures must be passed an implementation that matches all of them
// Inferences are made quadratic-pairwise to and from these overload sets
var NonGenericParameter;
(function(NonGenericParameter) {
    var foo4 = function foo4(cb) {
        return new cb(null);
    };
    var a;
    var r = foo4(a);
    var b;
    var r2 = foo4(b);
})(NonGenericParameter || (NonGenericParameter = {}));
var GenericParameter;
(function(GenericParameter) {
    var foo5 = function foo5(cb) {
        return cb;
    };
    var foo6 = function foo6(cb) {
        return cb;
    };
    var foo7 = function foo7(x, cb) {
        return cb;
    };
    var a;
    var r5 = foo5(a); // new{} => string; new(x:number) => {}
    var b;
    var r7 = foo5(b); // new any => string; new(x:number) => any
    var r8 = foo6(a); // error
    var r9 = foo6(b); // new any => string; new(x:any, y?:any) => string
    var r13 = foo7(1, b); // new any => string; new(x:any, y?:any) => string
    var c;
    var c2;
    var r14 = foo7(1, c); // new any => string; new(x:any, y?:any) => string
    var r15 = foo7(1, c2); // new any => string; new(x:any, y?:any) => string
})(GenericParameter || (GenericParameter = {}));
