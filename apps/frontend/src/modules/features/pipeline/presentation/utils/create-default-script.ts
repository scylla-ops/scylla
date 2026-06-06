export const createDefaultScript = (projectId: string) =>
  JSON.stringify(
    {
      name: 'my-pipeline',
      projectId,
      nodes: [
        {
          id: 'welcome',
          deps: [],
          kind: 'script',
          shell: 'sh',
          script: 'echo "Welcome to Scylla!"',
          workingDir: '',
          env: {},
        },
        {
          id: 'example',
          deps: ['welcome'],
          kind: 'script',
          shell: 'sh',
          script: 'echo "This is an example pipeline."',
          workingDir: '',
          env: {},
        },
      ],
    },
    null,
    2,
  );
