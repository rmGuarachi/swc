//// [declarationEmitWorkWithInlineComments.ts]
import { _ as _class_call_check } from "@swc/helpers/_/_class_call_check";
export var Foo = function Foo(isInternal1, isInternal2, isInternal3, isInternal4, isInternal5, isInternal6 /* trailing */ , isInternal7, notInternal1, notInternal2, notInternal3) {
    _class_call_check(this, Foo), this.isInternal1 = isInternal1, this.isInternal2 = isInternal2, this.isInternal3 = isInternal3, this.isInternal4 = isInternal4, this.isInternal5 = isInternal5, this.isInternal6 = isInternal6, this.isInternal7 = isInternal7, this.notInternal1 = notInternal1, this.notInternal2 = notInternal2, this.notInternal3 = notInternal3;
};
export var Bar = function Bar(isInternal1) {
    _class_call_check(this, Bar), this.isInternal1 = isInternal1;
};
export var Baz = function Baz(isInternal) {
    _class_call_check(this, Baz), this.isInternal = isInternal;
};
