//// [class.ts]
var X, X1;
((X1 = X || (X = {})).Y || (X1.Y = {})).Point = class {
    constructor(x, y){
        this.x = x, this.y = y;
    }
};
//// [module.ts]
var X, X1, Y;
let Point;
(Point = (Y = (X1 = X || (X = {})).Y || (X1.Y = {})).Point || (Y.Point = {})).Origin = new Point(0, 0);
//// [test.ts]
//var cl: { x: number; y: number; }
new X.Y.Point(1, 1), X.Y.Point.Origin;
 // error not expected here same as bug 83996 ?
//// [simple.ts]
class A {
}
(A || (A = {})).Instance = new A(), A.Instance, new A();
