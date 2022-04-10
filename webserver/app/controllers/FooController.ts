import { Body, Controller, Get, Path, Post, Query, Route, Response, SuccessResponse } from 'tsoa';
import type { User } from '../models/Foo';
import type { ValidateErrorJSON } from '../models/ValidationErrorJSON';
import { UsersService, UserCreationParams } from '../services/FooService';

@Route('users')
export class UsersController extends Controller {
  @Get('{userId}')
  @Response<ValidateErrorJSON>(422, 'Validation Failed')
  @SuccessResponse('201', 'Created')
  public async getUser(
    @Path()
    userId: number,
    @Query()
    name?: string
  ): Promise<User> {
    return new UsersService().get(userId, name);
  }

  @SuccessResponse('201', 'Created') // Custom success response
  @Post()
  public async createUser(
    @Body()
    requestBody: UserCreationParams
  ): Promise<void> {
    this.setStatus(201); // set return status 201
    new UsersService().create(requestBody);
    return;
  }
}
