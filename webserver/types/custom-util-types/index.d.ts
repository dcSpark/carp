declare type Equals<X, Y> = (<T>() => T extends X ? 1 : 2) extends <
  T
>() => T extends Y ? 1 : 2
  ? true
  : false;

declare type NonNullableOrVoidable<T> = T extends void | null | undefined
  ? never
  : T;
declare type WithNonNullableFields<T> = {
  [P in keyof T]: NonNullableOrVoidable<T[P]>;
};
