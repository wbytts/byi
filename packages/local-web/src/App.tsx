import { useEffect, useState } from 'react';
import {
  Alert,
  Button,
  Descriptions,
  Divider,
  Form,
  Input,
  Result,
  Select,
  Space,
  Tag,
  Typography,
} from 'antd';
import {
  CloudSyncOutlined,
  GithubOutlined,
  HomeOutlined,
  ReloadOutlined,
  ThunderboltOutlined,
} from '@ant-design/icons';
import { useServerStore } from './store';
import type { GitHubConfigRequest, SyncRemote, WebDavConfigRequest } from './api';

const { Text, Title } = Typography;
type ActiveView = 'home' | 'sync';

export function App() {
  const [activeView, setActiveView] = useState<ActiveView>('home');
  const [githubForm] = Form.useForm<GitHubConfigRequest>();
  const [webdavForm] = Form.useForm<WebDavConfigRequest>();
  const {
    syncStatus,
    lastOperation,
    syncLoading,
    actionLoading,
    syncError,
    loadSyncStatus,
    configureGitHub,
    configureWebDav,
    runSyncOperation,
  } = useServerStore();

  useEffect(() => {
    if (activeView === 'sync') {
      void loadSyncStatus();
    }
  }, [activeView, loadSyncStatus]);

  useEffect(() => {
    if (syncStatus?.remote?.provider === 'github') {
      githubForm.setFieldsValue({
        repo: syncStatus.remote.repo,
        branch: syncStatus.remote.branch,
        base_path: syncStatus.remote.base_path,
      });
    }

    if (syncStatus?.remote?.provider === 'webdav') {
      webdavForm.setFieldsValue({
        preset: syncStatus.remote.preset,
        url: syncStatus.remote.endpoint_url,
        username: syncStatus.remote.username,
        base_path: syncStatus.remote.base_path,
      });
    }
  }, [githubForm, syncStatus, webdavForm]);

  return (
    <main className="flex min-h-screen bg-slate-50 text-slate-950">
      <aside className="flex w-64 shrink-0 flex-col border-r border-slate-200 bg-white">
        <div className="border-b border-slate-200 px-5 py-4">
          <Space align="center" size={12}>
            <span className="flex h-9 w-9 items-center justify-center rounded-md bg-blue-600 text-white">
              <ThunderboltOutlined />
            </span>
            <div>
              <Title level={5} className="!mb-0">
                BYI
              </Title>
              <Text type="secondary">本地控制台</Text>
            </div>
          </Space>
        </div>

        <nav className="flex flex-1 flex-col gap-2 p-4">
          <Button
            block
            className="!justify-start"
            icon={<HomeOutlined />}
            type={activeView === 'home' ? 'primary' : 'text'}
            onClick={() => setActiveView('home')}
          >
            首页
          </Button>

          <Button
            block
            className="!justify-start"
            icon={<CloudSyncOutlined />}
            type={activeView === 'sync' ? 'primary' : 'text'}
            onClick={() => setActiveView('sync')}
          >
            同步
          </Button>

          {activeView === 'sync' && syncError ? (
            <>
              <Divider className="!my-2" />
              <Alert type="error" showIcon message={syncError} />
            </>
          ) : null}
        </nav>
      </aside>

      <section className="min-w-0 flex-1 px-8 py-6">
        {activeView === 'home' ? null : (
          <SyncPanel
            syncStatus={syncStatus}
            lastOperation={lastOperation}
            syncLoading={syncLoading}
            actionLoading={actionLoading}
            githubForm={githubForm}
            webdavForm={webdavForm}
            onRefresh={loadSyncStatus}
            onConfigureGitHub={configureGitHub}
            onConfigureWebDav={configureWebDav}
            onRunSyncOperation={runSyncOperation}
          />
        )}
      </section>
    </main>
  );
}

function SyncPanel({
  syncStatus,
  lastOperation,
  syncLoading,
  actionLoading,
  githubForm,
  webdavForm,
  onRefresh,
  onConfigureGitHub,
  onConfigureWebDav,
  onRunSyncOperation,
}: {
  syncStatus: ReturnType<typeof useServerStore.getState>['syncStatus'];
  lastOperation: ReturnType<typeof useServerStore.getState>['lastOperation'];
  syncLoading: boolean;
  actionLoading: ReturnType<typeof useServerStore.getState>['actionLoading'];
  githubForm: ReturnType<typeof Form.useForm<GitHubConfigRequest>>[0];
  webdavForm: ReturnType<typeof Form.useForm<WebDavConfigRequest>>[0];
  onRefresh: () => Promise<void>;
  onConfigureGitHub: (request: GitHubConfigRequest) => Promise<void>;
  onConfigureWebDav: (request: WebDavConfigRequest) => Promise<void>;
  onRunSyncOperation: (action: 'test' | 'pull' | 'push') => Promise<void>;
}) {
  return (
    <>
      <div className="mb-6 flex items-center justify-between gap-4">
        <div>
          <Title level={3} className="!mb-0">
            同步
          </Title>
          <Text type="secondary">配置远端存储，并执行测试、拉取和推送操作。</Text>
        </div>
        <Button icon={<ReloadOutlined />} loading={syncLoading} onClick={() => void onRefresh()}>
          刷新
        </Button>
      </div>

      <section className="space-y-5">
          <section className="rounded-md border border-slate-200 bg-white p-5">
            {syncStatus ? (
              <Descriptions title="远端状态" bordered column={1} size="middle">
                <Descriptions.Item label="状态">
                  <Tag color={syncStatus.configured ? 'success' : 'warning'}>
                    {syncStatus.configured ? '已配置' : '未配置'}
                  </Tag>
                </Descriptions.Item>
                {syncStatus.remote ? <RemoteDescriptions remote={syncStatus.remote} /> : null}
                <Descriptions.Item label="消息">
                  <pre className="m-0 whitespace-pre-wrap font-mono text-xs leading-5">
                    {syncStatus.message}
                  </pre>
                </Descriptions.Item>
              </Descriptions>
            ) : (
              <Result status="info" title="正在加载同步状态" />
            )}

            {lastOperation ? (
              <Alert
                className="mt-4"
                type="success"
                showIcon
                message={lastOperation.message}
              />
            ) : null}

            <Divider />

            <div className="grid gap-5 xl:grid-cols-2">
              <section>
                <Space align="center" className="mb-3">
                  <GithubOutlined className="text-blue-600" />
                  <Text strong>GitHub</Text>
                </Space>
                <Form
                  form={githubForm}
                  layout="vertical"
                  initialValues={{ branch: 'main', base_path: '.byi' }}
                  onFinish={(values) => void onConfigureGitHub(values)}
                >
                  <Form.Item
                    name="repo"
                    label="仓库"
                    rules={[{ required: true, message: '请使用 owner/repo 格式。' }]}
                  >
                    <Input placeholder="owner/repo" />
                  </Form.Item>
                  <Form.Item name="branch" label="分支">
                    <Input placeholder="main" />
                  </Form.Item>
                  <Form.Item name="base_path" label="基础路径">
                    <Input placeholder=".byi" />
                  </Form.Item>
                  <Button
                    type="primary"
                    htmlType="submit"
                    icon={<GithubOutlined />}
                    loading={actionLoading === 'github'}
                  >
                    保存 GitHub 远端
                  </Button>
                </Form>
              </section>

              <section>
                <Space align="center" className="mb-3">
                  <CloudSyncOutlined className="text-blue-600" />
                  <Text strong>WebDAV</Text>
                </Space>
                <Form
                  form={webdavForm}
                  layout="vertical"
                  initialValues={{ preset: 'jianguoyun', base_path: '.byi' }}
                  onFinish={(values) => void onConfigureWebDav(values)}
                >
                  <Form.Item name="preset" label="预设">
                    <Select
                      options={[
                        { value: 'jianguoyun', label: '坚果云' },
                        { value: 'custom', label: '自定义' },
                      ]}
                    />
                  </Form.Item>
                  <Form.Item shouldUpdate noStyle>
                    {({ getFieldValue }) =>
                      getFieldValue('preset') === 'custom' ? (
                        <Form.Item
                          name="url"
                          label="WebDAV URL"
                          rules={[{ required: true, message: '自定义 WebDAV 需要 URL。' }]}
                        >
                          <Input placeholder="https://example.com/dav/" />
                        </Form.Item>
                      ) : null
                    }
                  </Form.Item>
                  <Form.Item name="username" label="用户名">
                    <Input placeholder="name@example.com" />
                  </Form.Item>
                  <Form.Item name="base_path" label="基础路径">
                    <Input placeholder=".byi" />
                  </Form.Item>
                  <Button
                    type="primary"
                    htmlType="submit"
                    icon={<CloudSyncOutlined />}
                    loading={actionLoading === 'webdav'}
                  >
                    保存 WebDAV 远端
                  </Button>
                </Form>
              </section>
            </div>

            <Divider />

            <Space wrap>
              <Button
                icon={<CloudSyncOutlined />}
                disabled={!syncStatus?.configured}
                loading={actionLoading === 'test'}
                onClick={() => void onRunSyncOperation('test')}
              >
                测试
              </Button>
              <Button
                icon={<CloudSyncOutlined />}
                disabled={!syncStatus?.configured}
                loading={actionLoading === 'pull'}
                onClick={() => void onRunSyncOperation('pull')}
              >
                拉取
              </Button>
              <Button
                type="primary"
                icon={<CloudSyncOutlined />}
                disabled={!syncStatus?.configured}
                loading={actionLoading === 'push'}
                onClick={() => void onRunSyncOperation('push')}
              >
                推送
              </Button>
            </Space>
          </section>
        </section>
      </>
  );
}

function RemoteDescriptions({ remote }: { remote?: SyncRemote }) {
  if (!remote) {
    return null;
  }

  if (remote.provider === 'github') {
    return (
      <>
        <Descriptions.Item label="提供方">GitHub</Descriptions.Item>
        <Descriptions.Item label="仓库">{remote.repo}</Descriptions.Item>
        <Descriptions.Item label="分支">{remote.branch}</Descriptions.Item>
        <Descriptions.Item label="基础路径">{remote.base_path}</Descriptions.Item>
        <Descriptions.Item label="认证">{remote.auth}</Descriptions.Item>
      </>
    );
  }

  return (
    <>
      <Descriptions.Item label="提供方">WebDAV</Descriptions.Item>
      <Descriptions.Item label="预设">{remote.preset === 'jianguoyun' ? '坚果云' : '自定义'}</Descriptions.Item>
      <Descriptions.Item label="地址">{remote.endpoint_url}</Descriptions.Item>
      <Descriptions.Item label="用户名">{remote.username ?? '未设置'}</Descriptions.Item>
      <Descriptions.Item label="基础路径">{remote.base_path}</Descriptions.Item>
    </>
  );
}
