'use client';

import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { writeTextFile, readTextFile, mkdir, exists, BaseDirectory } from '@tauri-apps/plugin-fs';
import { open } from '@tauri-apps/plugin-shell';
import * as path from '@tauri-apps/api/path';
import { Button } from '../../ui/primitives/button';
import { Card, CardHeader, CardTitle, CardContent } from '../../ui/primitives/card';
import { Input } from '../../ui/primitives/input';
import { Label } from '../../ui/primitives/label';
import { Badge } from '../../ui/primitives/badge';
import { Alert, AlertDescription } from '../../ui/primitives/alert';
import { Trash2, Plus, RefreshCw, Zap, ZapOff, ExternalLink } from 'lucide-react';
import { bootstrapCheck, bootstrapInstall } from '../../lib/tauri/window';

interface FrpsConfig {
  server_addr: string;
  server_port: number;
  token?: string;
  user: string;
}

interface PortMapping {
  name: string;
  local_ip: string;
  local_port: number;
  remote_port: number;
  protocol: string;
  custom_domains?: string[];
  subdomain?: string;
}

interface SimplePortMapping {
  local_port: number;
}

interface FrpsStatus {
  connected: boolean;
  server_addr: string;
  active_mappings: PortMapping[];
  pid?: number;
  max_mappings: number;
  remaining_mappings: number;
}

interface PersistedState {
  config: FrpsConfig;
  status: FrpsStatus;
  newMapping: SimplePortMapping;
}

export default function FrpsManager() {
  const [config, setConfig] = useState<FrpsConfig>({
    server_addr: '64.23.133.199',
    server_port: 7000,
    token: '',
    user: '',
  });

  const [status, setStatus] = useState<FrpsStatus>({
    connected: false,
    server_addr: '',
    active_mappings: [],
    max_mappings: 3,
    remaining_mappings: 3,
  });

  const [newMapping, setNewMapping] = useState<SimplePortMapping>({
    local_port: 3000,
  });

  const [message, setMessage] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [showNewMapping, setShowNewMapping] = useState(false);
  const [isBootstrapped, setIsBootstrapped] = useState(false);
  const [isBootstrapping, setIsBootstrapping] = useState(false);
  const [bootstrapError, setBootstrapError] = useState('');
  const [hasLoadedState, setHasLoadedState] = useState(false);

  // Fixed: Use correct file path without duplication
  const STATE_FILE_PATH = 'frps_state.json';

  // Ensure the AppData directory exists
  const ensureAppDataDir = useCallback(async () => {
    try {
      await mkdir('', {
        baseDir: BaseDirectory.AppData,
        recursive: true,
      });
      console.log('AppData directory ensured');
    } catch (error) {
      console.error('Failed to ensure AppData directory:', error);
    }
  }, []);

  // Initialize default state file if it doesn't exist
  const initializeStateFile = useCallback(async () => {
    try {
      const fileExists = await exists(STATE_FILE_PATH, { baseDir: BaseDirectory.AppData });
      if (!fileExists) {
        const defaultState: PersistedState = {
          config: {
            server_addr: '64.23.133.199',
            server_port: 7000,
            token: '',
            user: '',
          },
          status: {
            connected: false,
            server_addr: '',
            active_mappings: [],
            max_mappings: 3,
            remaining_mappings: 3,
          },
          newMapping: {
            local_port: 3000,
          },
        };
        
        await writeTextFile(STATE_FILE_PATH, JSON.stringify(defaultState, null, 2), {
          baseDir: BaseDirectory.AppData,
        });
        console.log('Initialized default state file');
      }
    } catch (error) {
      console.error('Failed to initialize state file:', error);
    }
  }, []);

  // Load persisted state from file
  const loadPersistedState = useCallback(async () => {
    try {
      await ensureAppDataDir();
      await initializeStateFile();
      
      const stateJson = await readTextFile(STATE_FILE_PATH, { baseDir: BaseDirectory.AppData });
      const persistedState: PersistedState = JSON.parse(stateJson);
      
      setConfig(persistedState.config);
      setStatus(persistedState.status);
      setNewMapping(persistedState.newMapping);
      setMessage('State loaded successfully');
      setHasLoadedState(true);
    } catch (error) {
      console.error('Failed to load persisted state:', error);
      setMessage('Failed to load persisted state; using default configuration');
      setHasLoadedState(true);
    }
  }, [ensureAppDataDir, initializeStateFile]);

  // Save state to file
  const savePersistedState = useCallback(async () => {
    try {
      await ensureAppDataDir();
      const state: PersistedState = { config, status, newMapping };
      await writeTextFile(STATE_FILE_PATH, JSON.stringify(state, null, 2), {
        baseDir: BaseDirectory.AppData,
      });
      console.log('State saved successfully');
    } catch (error) {
      console.error('Failed to save persisted state:', error);
      setMessage('Failed to save persisted state');
    }
  }, [config, status, newMapping, ensureAppDataDir]);

  const checkBootstrap = useCallback(async () => {
    setMessage('Checking dependencies...');
    try {
      const isInstalled = await bootstrapCheck();
      if (isInstalled) {
        setIsBootstrapped(true);
        setMessage('✓ Dependencies ready');
      } else {
        setMessage('⚠ Installing dependencies...');
        await installBootstrap();
      }
    } catch (error) {
      setBootstrapError(`${error}`);
      setMessage(`❌ Bootstrap error: ${error}`);
    }
  }, []);

  const installBootstrap = useCallback(async () => {
    try {
      setIsBootstrapping(true);
      setMessage('Installing dependencies...');
      await bootstrapInstall();
      setIsBootstrapped(true);
      setMessage('✓ Installation complete');
    } catch (error) {
      setBootstrapError(`${error}`);
      setMessage(`❌ Install failed: ${error}`);
    } finally {
      setIsBootstrapping(false);
    }
  }, []);

  useEffect(() => {
    const loadConfig = async () => {
      try {
        const loadedConfig = await invoke<FrpsConfig>('frps_load_config');
        setConfig(loadedConfig);
      } catch (error) {
        console.error('Failed to load config:', error);
        setMessage('Failed to load config');
      }
      await checkBootstrap();
      await loadPersistedState();
      await checkStatus();
    };
    loadConfig();
  }, [checkBootstrap, loadPersistedState]);

  useEffect(() => {
    if (hasLoadedState) {
      savePersistedState();
    }
  }, [config, status, newMapping, hasLoadedState, savePersistedState]);

  const checkStatus = useCallback(async () => {
    if (!isBootstrapped) return;
    try {
      const result = await invoke<FrpsStatus>('frps_get_status');
      setStatus(result);
    } catch (error) {
      console.error('Failed to get status:', error);
      setMessage('Failed to get status');
    }
  }, [isBootstrapped]);

  const handleConnect = useCallback(async () => {
    if (!isBootstrapped) {
      setMessage('Dependencies not ready');
      return;
    }
    setIsLoading(true);
    try {
      const result = await invoke<string>('frps_connect');
      setMessage(result);
      await checkStatus();
    } catch (error) {
      setMessage(error as string);
    } finally {
      setIsLoading(false);
    }
  }, [isBootstrapped, checkStatus]);

  const handleDisconnect = useCallback(async () => {
    if (!isBootstrapped) {
      setMessage('Dependencies not ready');
      return;
    }
    setIsLoading(true);
    try {
      const result = await invoke<string>('frps_disconnect');
      setMessage(result);
      await checkStatus();
    } catch (error) {
      setMessage(error as string);
    } finally {
      setIsLoading(false);
    }
  }, [isBootstrapped, checkStatus]);

  const handleTestConnection = useCallback(async () => {
    if (!isBootstrapped) {
      setMessage('Dependencies not ready');
      return;
    }
    setIsLoading(true);
    try {
      const result = await invoke<string>('frps_test_connection');
      setMessage(result);
    } catch (error) {
      setMessage(error as string);
    } finally {
      setIsLoading(false);
    }
  }, [isBootstrapped]);

  const handleAddMapping = useCallback(async () => {
    if (!isBootstrapped) {
      setMessage('Dependencies not ready');
      return;
    }
    if (!newMapping.local_port) {
      setMessage('Please enter local port');
      return;
    }
    if (status.remaining_mappings === 0) {
      setMessage(`Maximum number of port mappings (${status.max_mappings}) reached. Please remove existing mappings before adding new ones.`);
      return;
    }

    setIsLoading(true);
    try {
      const result = await invoke<string>('frps_add_port_mapping', {
        simpleMapping: newMapping,
      });
      setMessage(result);
      setShowNewMapping(false);
      setNewMapping({ local_port: 3000 });
      await checkStatus();
    } catch (error) {
      setMessage(error as string);
    } finally {
      setIsLoading(false);
    }
  }, [isBootstrapped, newMapping, status.max_mappings, status.remaining_mappings, checkStatus]);

  const handleRemoveMapping = useCallback(
    async (mappingName: string) => {
      if (!isBootstrapped) {
        setMessage('Dependencies not ready');
        return;
      }
      setIsLoading(true);
      try {
        const result = await invoke<string>('frps_remove_port_mapping', {
          mappingName,
        });
        setMessage(result);
        await checkStatus();
      } catch (error) {
        setMessage(error as string);
      } finally {
        setIsLoading(false);
      }
    },
    [isBootstrapped, checkStatus]
  );


  const openExternalLink = useCallback(async (url: string) => {
    try {
      await open(url);
    } catch (error) {
      console.error('Failed to open URL:', error);
      setMessage('Failed to open URL');
    }
  }, []);

  return (
    <div className="container mx-auto p-6 space-y-6">
      <h1 className="text-3xl font-bold">FRPS Port Mapping Manager</h1>

      {message && (
        <Alert
          className={
            message.includes('success') || message.includes('ready') || message.includes('complete')
              ? 'border-green-500'
              : 'border-red-500'
          }
        >
          <AlertDescription>{message}</AlertDescription>
        </Alert>
      )}

      {/* Connection Status */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            Connection Status
            <Badge variant={status.connected ? 'default' : 'secondary'}>
              {status.connected ? 'Connected' : 'Disconnected'}
            </Badge>
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div className="text-sm text-gray-600">
              <p>
                <strong>Server:</strong> {config.server_addr}:{config.server_port}
              </p>
              <p>
                <strong>Protocol:</strong> TCP
              </p>
              <p>
                <strong>Local IP:</strong> 127.0.0.1
              </p>
              <p>
                <strong>Port Mappings:</strong> {status.active_mappings.length} / {status.max_mappings}
              </p>
              <p>
                <strong>Dependencies:</strong>{' '}
                {isBootstrapped ? 'Ready' : isBootstrapping ? 'Installing...' : 'Not Ready'}
              </p>
            </div>
            <div className="flex items-center gap-4">
              <Button
                onClick={status.connected ? handleDisconnect : handleConnect}
                disabled={isLoading || !isBootstrapped}
                className={status.connected ? 'bg-red-500 hover:bg-red-600' : ''}
              >
                {isLoading ? (
                  <RefreshCw className="h-4 w-4 mr-2 animate-spin" />
                ) : status.connected ? (
                  <ZapOff className="h-4 w-4 mr-2" />
                ) : (
                  <Zap className="h-4 w-4 mr-2" />
                )}
                {status.connected ? 'Disconnect' : 'Connect'}
              </Button>
              <Button onClick={checkStatus} variant="outline" disabled={!isBootstrapped}>
                <RefreshCw className="h-4 w-4 mr-2" />
                Refresh
              </Button>
              <Button onClick={handleTestConnection} variant="outline" disabled={!isBootstrapped}>
                Test Connection
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Port Mappings */}
      <Card>
        <CardHeader>
  <CardTitle className="flex items-center justify-between w-full">
    <span>Port Mappings</span>
    <div className="flex items-center gap-2">
      {status.remaining_mappings === 0 && (
        <Alert className="border-red-500">
          <AlertDescription>
            Maximum of {status.max_mappings} port mappings reached.
          </AlertDescription>
        </Alert>
      )}
      <Button
        onClick={() => setShowNewMapping(true)}
        size="sm"
        disabled={!status.connected || !isBootstrapped || status.remaining_mappings === 0}
      >
        <Plus className="h-4 w-4 mr-2" />
        Add Mapping
      </Button>
    </div>
  </CardTitle>
</CardHeader>

        <CardContent>
          {status.active_mappings.length === 0 ? (
            <p className="text-gray-500 text-center py-8">No port mappings configured</p>
          ) : (
            <div className="space-y-2">
              {status.active_mappings.map((mapping) => (
                <div
                  key={mapping.name}
                  className="flex items-center justify-between p-3 border rounded-lg"
                >
                  <div className="flex items-center gap-4">
                    <Badge variant="outline">{mapping.protocol.toUpperCase()}</Badge>
                    <span className="font-medium">{mapping.name}</span>
                    <span className="text-sm text-gray-500">
                      Local: {mapping.local_port} → Remote: {mapping.remote_port}
                    </span>
                  </div>
                  <div className="flex items-center gap-2">
                    {status.connected && (
                      <Button
                        onClick={() =>
                          openExternalLink(`http://${config.server_addr}:${mapping.remote_port}/`)
                        }
                        variant="outline"
                        size="sm"
                        className="text-blue-500 hover:text-blue-700"
                      >
                        <ExternalLink className="h-4 w-4 mr-1" />
                        Open
                      </Button>
                    )}
                    <Button
                      onClick={() => handleRemoveMapping(mapping.name)}
                      variant="outline"
                      size="sm"
                      className="text-red-500 hover:text-red-700"
                      disabled={!isBootstrapped}
                    >
                      <Trash2 className="h-4 w-4" />
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      {/* New Mapping Modal */}
      {showNewMapping && (
        <Card>
          <CardHeader>
            <CardTitle>Add New Port Mapping</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="text-sm text-gray-600 p-3 bg-gray-50 rounded">
              <p>
                <strong>Auto-configured settings:</strong>
              </p>
              <p>
                • Server: {config.server_addr}:{config.server_port}
              </p>
              <p>• Protocol: TCP</p>
              <p>• Local IP: 127.0.0.1</p>
              <p>• Remote Port: Auto-allocated (8000-8010)</p>
              <p>• Name: Will be auto-generated (e.g., nextjs3000)</p>
              <p>
                <strong>Dependencies:</strong> {isBootstrapped ? 'Ready' : 'Not Ready'}
              </p>
              <p>
                <strong>Remaining Mappings:</strong> {status.remaining_mappings} / {status.max_mappings}
              </p>
            </div>
            <div className="grid grid-cols-1 gap-4">
              <div>
                <Label htmlFor="local_port">Local Port</Label>
                <Input
                  id="local_port"
                  type="number"
                  value={newMapping.local_port}
                  onChange={(e) =>
                    setNewMapping({ ...newMapping, local_port: parseInt(e.target.value) || 0 })
                  }
                  placeholder="3000"
                  disabled={!isBootstrapped}
                />
                <p className="text-xs text-gray-500 mt-1">
                  Enter the local port of your application (e.g., 3000 for Next.js)
                </p>
              </div>
            </div>
            <div className="flex gap-2">
              <Button onClick={handleAddMapping} disabled={isLoading || !isBootstrapped || status.remaining_mappings === 0}>
                {isLoading ? <RefreshCw className="h-4 w-4 mr-2 animate-spin" /> : null}
                Add Mapping
              </Button>
              <Button onClick={() => setShowNewMapping(false)} variant="outline">
                Cancel
              </Button>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}