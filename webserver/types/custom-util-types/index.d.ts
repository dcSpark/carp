declare type Equals<X, Y> = (<T>() => T extends X ? 1 : 2) extends <
  T
>() => T extends Y ? 1 : 2
  ? true
  : false;

declare type InexactSubset<T> = { [P in keyof T]?: T[P] };
declare type NonNullableOrVoidable<T> = T extends void | null | undefined
  ? never
  : T;
declare type WithNonNullableFields<T> = {
  [P in keyof T]: NonNullableOrVoidable<T[P]>;
};

declare type ConvertBuffer<T> = T extends Buffer ? string : T;
declare type BuffersToStrings<T> = {
  [P in keyof T]: ConvertBuffer<T[P]>;
};
declare type WithoutDatabaseId<T> = Omit<T, "id">;
