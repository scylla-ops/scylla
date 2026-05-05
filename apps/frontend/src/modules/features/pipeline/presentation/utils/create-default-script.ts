export const createDefaultScript = (projectId: string) =>
  JSON.stringify(
    {
      name: 'my-pipeline',
      projectId,
      nodes: [
        { id: 'build', deps: [], command: 'cargo', args: ['build', '--release'] },
        { id: 'test', deps: ['build'], command: 'cargo', args: ['test'] },
      ],
    },
    null,
    2,
  );
