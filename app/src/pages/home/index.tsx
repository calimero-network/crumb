import React, { useEffect, useState } from 'react';
import styled from 'styled-components';
import { useNavigate } from 'react-router-dom';
import {
  clientLogout,
  getAccessToken,
  getAppEndpointKey,
  getApplicationId,
  getContextId,
  getRefreshToken,
  NodeEvent,
  ResponseData,
  SubscriptionsClient,
} from '@calimero-network/calimero-client';

import {
  ClientApiDataSource,
  getWsSubscriptionsClient,
} from '../../api/dataSource/ClientApiDataSource';
import {
  GetCountResponse,
  IncreaseCountRequest,
  IncreaseCountResponse,
  ResetCounterResponse,
} from '../../api/clientApi';

const FullPageCenter = styled.div`
  display: flex;
  height: 100vh;
  width: 100vw;
  background-color: #111111;
  justify-content: center;
  align-items: center;
  flex-direction: column;
`;

const TextStyle = styled.div`
  color: white;
  margin-bottom: 2em;
  font-size: 2em;
`;

const Button = styled.div`
  color: white;
  padding: 0.25em 1em;
  border-radius: 8px;
  font-size: 2em;
  background: #5dbb63;
  cursor: pointer;
  justify-content: center;
  display: flex;
`;

const ButtonReset = styled.div`
  color: white;
  padding: 0.25em 1em;
  border-radius: 8px;
  font-size: 1em;
  background: #ffa500;
  cursor: pointer;
  justify-content: center;
  display: flex;
  margin-top: 1rem;
`;

const StatusTitle = styled.div`
  color: white;
  justify-content: center;
  display: flex;
`;

const StatusValue = styled.div`
  color: white;
  font-size: 60px;
  justify-content: center;
  display: flex;
`;

const LogoutButton = styled.div`
  color: black;
  margin-top: 2rem;
  padding: 0.25em 1em;
  border-radius: 8px;
  font-size: 1em;
  background: white;
  cursor: pointer;
  justify-content: center;
  display: flex;
`;

export default function HomePage() {
  const navigate = useNavigate();
  const url = getAppEndpointKey();
  const applicationId = getApplicationId();
  const accessToken = getAccessToken();
  const refreshToken = getRefreshToken();
  const [count, setCount] = useState<number | null>(null);

  useEffect(() => {
    if (!url || !applicationId || !accessToken || !refreshToken) {
      navigate('/auth');
    }
  }, [accessToken, applicationId, navigate, refreshToken, url]);

  async function increaseCounter() {
    const params: IncreaseCountRequest = {
      count: 1,
    };
    const result: ResponseData<IncreaseCountResponse> =
      await new ClientApiDataSource().increaseCount(params);
    if (result?.error) {
      console.error('Error:', result.error);
      window.alert(`${result.error.message}`);
      return;
    }
    await getCount();
  }

  async function getCount() {
    const result: ResponseData<GetCountResponse> =
      await new ClientApiDataSource().getCount();
    if (result?.error) {
      console.error('Error:', result.error);
      window.alert(`${result.error.message}`);
      return;
    }
    if (result.data.count || result.data.count === 0) {
      setCount(Number(result.data.count));
    }
  }

  async function resetCount() {
    const result: ResponseData<ResetCounterResponse> =
      await new ClientApiDataSource().reset();
    if (result?.error) {
      console.error('Error:', result.error);
      window.alert(`${result.error.message}`);
      return;
    }
    await getCount();
  }

  useEffect(() => {
    if (accessToken) {
      getCount();
    }
  }, [accessToken]);

  const observeNodeEvents = async () => {
    let subscriptionsClient: SubscriptionsClient = getWsSubscriptionsClient();
    await subscriptionsClient.connect();
    subscriptionsClient.subscribe([getContextId() ?? '']);

    subscriptionsClient?.addCallback((data: NodeEvent) => {
      if (
        'events' in data.data &&
        Array.isArray(data.data.events) &&
        data.data.events.length > 0
      ) {
        const event = data.data.events[0];
        if (event.data && Array.isArray(event.data)) {
          let currentValue = String.fromCharCode(...event.data);
          let currentValueInt = parseInt(currentValue);
          if (isNaN(currentValueInt)) {
            currentValueInt = 0;
          }
          setCount(currentValueInt);
        }
      }
    });
  };

  useEffect(() => {
    observeNodeEvents();
  }, []);

  const logout = () => {
    clientLogout();
    navigate('/');
  };

  return (
    <FullPageCenter>
      <TextStyle>
        <span> Welcome to home page!</span>
      </TextStyle>

      <StatusTitle> Current count is:</StatusTitle>
      <StatusValue> {count ?? '-'}</StatusValue>
      <Button onClick={increaseCounter}> + 1</Button>
      <ButtonReset onClick={resetCount}> Reset</ButtonReset>
      <LogoutButton onClick={() => navigate('/context')}>
        Context Actions
      </LogoutButton>
      <LogoutButton onClick={logout}>Logout</LogoutButton>
    </FullPageCenter>
  );
}
