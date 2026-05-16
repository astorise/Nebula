import * as vscode from "vscode";
import { NebulaDashboardProvider } from "./dashboardProvider";

export function activate(context: vscode.ExtensionContext): void {
  const dashboard = new NebulaDashboardProvider(context);
  const command = vscode.commands.registerCommand("nebula.openDashboard", () => {
    dashboard.open();
  });

  context.subscriptions.push(command, dashboard);
}

export function deactivate(): void {
}
