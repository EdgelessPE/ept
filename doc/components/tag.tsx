import type { ReactNode } from "react";
import "./index.css";

export const Tag = ({ children }: { children: ReactNode }) => {
  return <span className="tag">{children}</span>;
};
