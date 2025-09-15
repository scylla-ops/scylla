import './App.css';
import CoreRoot from '@/modules/core/presentation/ui/CoreRoot.tsx';
import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport';
import { UserServiceClient } from '@/generated/user.client.ts';
import { AuthServiceClient } from '@/generated/auth.client.ts';
import * as process from 'node:process';

function App() {
  /*const apiUrl = process.env.API_URL;
  const transport = new GrpcWebFetchTransport({
    baseUrl: apiUrl,
    format: 'binary',
  });
  const personClient = new UserServiceClient(transport);
  const authClient = new AuthServiceClient(transport);*/
  return <CoreRoot />;
}

export default App;
