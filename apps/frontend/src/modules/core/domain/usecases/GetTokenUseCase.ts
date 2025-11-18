import type { CoreRepository } from '@core/domain/repository/CoreRepository.ts';

export class GetTokenUseCase {
  constructor(private readonly repository: CoreRepository) {}
  execute(): string | null {
    return this.repository.getToken();
  }
}
