export const createDefaultScript = (projectId: string) =>
  JSON.stringify(
    {
      name: 'my-pipeline',
      projectId,
      nodes: [
        { id: 'welcome', deps: [], command: 'echo', args: ['Welcome to Scylla!'] },
        {
          id: 'example',
          deps: ['welcome'],
          command: 'echo',
          args: ['This is an example pipeline.'],
        },
      ],
    },
    null,
    2,
  );
