import * as path from 'path';
import { ExtensionContext, workspace } from 'vscode';

import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  Executable,
  TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: ExtensionContext) {
  const workspaceFolders = workspace.workspaceFolders;
  if (!workspaceFolders || workspaceFolders.length === 0) {
    return; // Workspace root required for this temporary setup
  }

  const workspaceRoot = workspaceFolders[0].uri.fsPath;
  const executablePath = 'peel';
  
  const executable: Executable = {
    command: executablePath,
    args: ['lsp'],
    options: {
      cwd: workspaceRoot,
    }
  };

  const serverOptions: ServerOptions = {
    run: executable,
    debug: executable,
  };

  // Options to control the language client
  const clientOptions: LanguageClientOptions = {
    // Register the server for peel documents
    documentSelector: [{ scheme: 'file', language: 'peel' }],
    synchronize: {
      // Notify the server about file changes to '.clientrc files contained in the workspace
      fileEvents: workspace.createFileSystemWatcher('**/.clientrc')
    }
  };

  // Create the language client and start the client.
  client = new LanguageClient(
    'peelLanguageServer',
    'Peel Language Server',
    serverOptions,
    clientOptions
  );

  // Start the client. This will also launch the server
  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
