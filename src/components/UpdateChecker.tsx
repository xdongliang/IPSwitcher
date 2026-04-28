import { useState, useEffect, useCallback, useRef } from "react";
import { check, type Update } from "@tauri-apps/plugin-updater";

type UpdateStage = "idle" | "available" | "downloading" | "done" | "restarting";

export default function UpdateChecker() {
  const [stage, setStage] = useState<UpdateStage>("idle");
  const [update, setUpdate] = useState<Update | null>(null);
  const [progress, setProgress] = useState(0);
  const checkedRef = useRef(false);

  useEffect(() => {
    if (checkedRef.current) return;
    checkedRef.current = true;

    const timer = setTimeout(async () => {
      try {
        const result = await check();
        if (result?.available) {
          setUpdate(result);
          setStage("available");
        }
      } catch (e) {
        console.error("Update check failed:", e);
      }
    }, 3000);

    return () => clearTimeout(timer);
  }, []);

  const handleUpdate = useCallback(async () => {
    if (!update) return;
    setStage("downloading");
    setProgress(0);

    try {
      let downloaded = 0;
      let contentLength = 0;

      await update.downloadAndInstall((event) => {
        switch (event.event) {
          case "Started":
            contentLength = event.data.contentLength ?? 0;
            break;
          case "Progress":
            downloaded += event.data.chunkLength;
            if (contentLength > 0) {
              setProgress(Math.min(Math.round((downloaded / contentLength) * 100), 100));
            }
            break;
          case "Finished":
            setProgress(100);
            break;
        }
      });

      setStage("done");
    } catch (e) {
      console.error("Update download failed:", e);
      setStage("available");
    }
  }, [update]);

  const handleRestart = useCallback(async () => {
    setStage("restarting");
    try {
      const { relaunch } = await import("@tauri-apps/plugin-process");
      await relaunch();
    } catch (e) {
      console.error("Relaunch failed:", e);
      // Fallback: prompt user to restart manually
      setStage("done");
    }
  }, []);

  const handleDismiss = useCallback(() => {
    setStage("idle");
    setUpdate(null);
  }, []);

  if (stage === "idle") return null;

  return (
    <div className="update-overlay">
      <div className="update-dialog">
        <div className="update-dialog-title">
          {stage === "available" && "发现新版本"}
          {stage === "downloading" && "正在下载更新"}
          {stage === "done" && "更新已就绪"}
          {stage === "restarting" && "正在重启…"}
        </div>

        <div className="update-dialog-body">
          {stage === "available" && (
            <>
              <p className="update-version">
                新版本：<strong>v{update?.version}</strong>
              </p>
              {update?.body && (
                <div className="update-notes">
                  <p className="update-notes-label">更新说明：</p>
                  <div className="update-notes-content">{update.body}</div>
                </div>
              )}
            </>
          )}

          {stage === "downloading" && (
            <div className="update-progress">
              <div className="update-progress-bar">
                <div
                  className="update-progress-fill"
                  style={{ width: `${progress}%` }}
                />
              </div>
              <p className="update-progress-text">{progress}%</p>
            </div>
          )}

          {stage === "done" && (
            <p>更新已下载并安装完成，请重启应用以使用新版本。</p>
          )}

          {stage === "restarting" && (
            <p>正在重启应用，请稍候…</p>
          )}
        </div>

        <div className="update-actions">
          {stage === "available" && (
            <>
              <button className="btn" onClick={handleDismiss}>
                稍后提醒
              </button>
              <button className="btn btn-primary" onClick={handleUpdate}>
                立即更新
              </button>
            </>
          )}

          {stage === "done" && (
            <button className="btn btn-primary" onClick={handleRestart}>
              安装并重启
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
