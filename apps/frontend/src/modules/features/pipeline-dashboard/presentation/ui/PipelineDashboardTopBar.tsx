import { Button } from '@shadcn';
import { useNavigate } from 'react-router-dom';

export const PipelineDashboardTopBar = () => {
  const navigate = useNavigate();
  return <Button onClick={() => navigate('pipeline-creation')}>New</Button>;
};
