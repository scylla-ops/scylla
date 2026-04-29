export const createDefaultScript = (projectId: string) =>
  `{\n"name": "my-pipeline",\n` +
  `"projectId": "${projectId}",\n` +
  `"nodes": [
    {
      "nodeId": "build",
      "deps": [],
      "command": "cargo",
      "args": ["build", "--release"]
    },
    {
      "nodeId": "test",
      "deps": ["build"],
      "command": "cargo",
      "args": ["test"]
    }
  ]\n` +
  `}`;
