import type { AuthzRepository } from '@/modules/features/authz/domain/repository/authz.repository.ts';

export class ListAuthzVocabularyUseCase {
  constructor(private readonly repository: AuthzRepository) {}

  public execute() {
    return this.repository.listAuthzVocabulary();
  }
}
