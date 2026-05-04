import { NODE_ID_FIELD_CREATE } from '@/modules/features/pipeline/presentation/utils/node-id-field.ts';

export const createDefaultScript = (projectId: string) =>
  JSON.stringify(
    {
      name: 'my-pipeline',
      projectId,
      nodes: [
        { [NODE_ID_FIELD_CREATE]: 'build', deps: [], command: 'cargo', args: ['build', '--release'] },
        { [NODE_ID_FIELD_CREATE]: 'test', deps: ['build'], command: 'cargo', args: ['test'] },
      ],
    },
    null,
    2,
  );
