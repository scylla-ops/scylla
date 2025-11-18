import type { CoreRepository } from '@core/domain/repository/CoreRepository.ts';

export class SetTokenUseCase {
  public constructor(private readonly repository: CoreRepository) {}

  public execute(token: string): void {
    this.repository.setToken(token);
  }
}
