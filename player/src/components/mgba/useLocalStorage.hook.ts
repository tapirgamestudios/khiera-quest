import { useCallback, useState } from "react";

export function useLocalStorage<T>(
  defaultValue: T,
  appName: string
): [T, (newValue: T) => void] {
  const [value, setValue] = useState(() => {
    try {
      const storageValue = localStorage.getItem(appName);
      if (storageValue) {
        return JSON.parse(storageValue);
      } else {
        return defaultValue;
      }
    } catch {
      return defaultValue;
    }
  });

  const setStoredValue = useCallback((newValue: T) => {
    setValue(newValue);
    try {
      localStorage.setItem(appName, JSON.stringify(newValue));
    } catch {}
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return [value, setStoredValue];
}
