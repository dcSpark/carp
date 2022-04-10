export interface ValidateErrorJSON {
  message: 'Validation failed';
  details: { [name: string]: unknown };
}
