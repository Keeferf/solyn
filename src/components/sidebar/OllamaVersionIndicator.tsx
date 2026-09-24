import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { FiRefreshCw, FiAlertCircle, FiCheckCircle } from "react-icons/fi";
import { useOllama } from "@/contexts/OllamaContext";

interface OllamaStatus {
  installed: boolean;
  running: boolean;
  version: string | null;
}

export const OllamaVersionIndicator = () => {
  const { status, refreshOllamaStatus } = useOllama();
  const [isOutdated, setIsOutdated] = useState(false);
  const [checking, setChecking] = useState(false);
  const [updating, setUpdating] = useState(false);
  const [latestVersion, setLatestVersion] = useState<string | null>(null);
  const [updateError, setUpdateError] = useState<string | null>(null);
  const [updateSuccess, setUpdateSuccess] = useState(false);
  const [progress, setProgress] = useState(0);

  // Check for updates whenever the status changes
  useEffect(() => {
    if (status?.running && status?.version) {
      checkForUpdates(status.version);
    } else {
      setIsOutdated(false);
      setLatestVersion(null);
    }
  }, [status?.running, status?.version]);

  // Listen for status updates from the backend
  useEffect(() => {
    const unlisten = listen<OllamaStatus>("ollama-status-update", (event) => {
      if (event.payload.running && event.payload.version) {
        checkForUpdates(event.payload.version);
        // If we were updating, check if the version changed
        if (updating) {
          const oldVersion = status?.version;
          if (oldVersion && event.payload.version !== oldVersion) {
            setUpdateSuccess(true);
            setUpdating(false);
            setIsOutdated(false);
            // Auto-hide success message after 5 seconds
            setTimeout(() => setUpdateSuccess(false), 5000);
          }
        }
      }
    });

    // Listen for download progress events
    const progressUnlisten = listen<any>("download-progress", (event) => {
      if (updating) {
        const progressData = event.payload;
        setProgress(progressData.percentage || 0);
      }
    });

    return () => {
      unlisten.then((fn) => fn());
      progressUnlisten.then((fn) => fn());
    };
  }, [updating, status?.version]);

  const checkForUpdates = async (currentVersion: string) => {
    if (checking) return;
    setChecking(true);
    setUpdateError(null);

    try {
      const response = await fetch(
        "https://api.github.com/repos/ollama/ollama/releases/latest",
      );

      if (!response.ok) {
        throw new Error("Failed to fetch latest version");
      }

      const data = await response.json();
      const latest = data.tag_name.replace("v", "");
      setLatestVersion(latest);

      const isOutdated = compareVersions(currentVersion, latest) < 0;
      setIsOutdated(isOutdated);
    } catch (error) {
      setIsOutdated(false);
    } finally {
      setChecking(false);
    }
  };

  const compareVersions = (v1: string, v2: string): number => {
    const parts1 = v1.split(".").map(Number);
    const parts2 = v2.split(".").map(Number);

    for (let i = 0; i < Math.max(parts1.length, parts2.length); i++) {
      const num1 = parts1[i] || 0;
      const num2 = parts2[i] || 0;

      if (num1 !== num2) {
        return num1 - num2;
      }
    }

    return 0;
  };

  const handleUpdate = async () => {
    if (!isOutdated || updating) return;

    setUpdating(true);
    setUpdateError(null);
    setUpdateSuccess(false);
    setProgress(0);

    try {

      // Add a timeout to the invoke call
      const updatePromise = invoke("update_ollama");
      const timeoutPromise = new Promise((_, reject) =>
        setTimeout(
          () => reject(new Error("Update timed out after 60 seconds")),
          60000,
        ),
      );

      await Promise.race([updatePromise, timeoutPromise]);


      // Wait a moment for the update to actually take effect
      await new Promise((resolve) => setTimeout(resolve, 3000));

      // Refresh the status with retry logic
      let retries = 0;
      const maxRetries = 8;
      let statusRefreshed = false;

      while (retries < maxRetries && !statusRefreshed) {
        try {
          await refreshOllamaStatus();
          statusRefreshed = true;
        } catch (error) {
          retries++;
          if (retries < maxRetries) {
            await new Promise((resolve) => setTimeout(resolve, 2000));
          }
        }
      }

      setUpdateSuccess(true);
      setUpdating(false);
      setIsOutdated(false);

      // Reset success state after 5 seconds
      setTimeout(() => setUpdateSuccess(false), 5000);
    } catch (error) {
      setUpdateError(error instanceof Error ? error.message : "Update failed");
      setUpdating(false);
      setIsOutdated(false);
    }
  };

  // Only show when there's an update available
  if (!isOutdated || !status?.installed || !status?.running) {
    if (updateSuccess) {
      return (
        <div className="flex items-center gap-1.5 px-2 py-1 bg-green-500/10 border border-green-500/20 rounded-lg text-xs">
          <FiCheckCircle className="w-3 h-3 text-green-500 shrink-0" />
          <span className="font-medium text-green-500">Updated!</span>
        </div>
      );
    }
    return null;
  }

  if (updateError) {
    return (
      <div className="flex items-center gap-1.5 px-2 py-1 bg-red-500/10 border border-red-500/20 rounded-lg text-xs">
        <FiAlertCircle className="w-3 h-3 text-red-500 shrink-0" />
        <span className="font-medium text-red-500">Update failed</span>
      </div>
    );
  }

  // Show update button
  return (
    <button
      onClick={handleUpdate}
      disabled={updating}
      className="flex items-center gap-1.5 px-2 py-1 bg-green-500/10 border border-green-500/20 rounded-lg hover:bg-green-500/20 transition-all group cursor-pointer text-xs min-w-20 justify-center"
      title={`Update Ollama from ${status.version} to ${latestVersion}`}
    >
      {updating ? (
        <>
          <FiRefreshCw className="w-3 h-3 text-green-500 animate-spin shrink-0" />
          <span className="font-medium text-green-500">
            {progress > 0 ? `${Math.round(progress)}%` : "Updating..."}
          </span>
        </>
      ) : (
        <>
          <FiAlertCircle className="w-3 h-3 text-green-500 shrink-0" />
          <span className="font-medium text-green-500 whitespace-nowrap">
            Update to {latestVersion}
          </span>
        </>
      )}
    </button>
  );
};
