/** Where the app's name and version come from: Tauri's app API, or a fake. */
export type AppInfo = {
  getName(): Promise<string>;
  getVersion(): Promise<string>;
};

/** Shows the app's name and version, such as "randomtask 0.2.1", in `footer`. */
export async function showAbout(footer: HTMLElement, app: AppInfo): Promise<void> {
  try {
    const [name, version] = await Promise.all([app.getName(), app.getVersion()]);
    footer.textContent = `${name} ${version}`;
  } catch {
    // Without its name or version, the footer stays empty rather than half-filled.
  }
}
