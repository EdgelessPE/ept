import type { ReactNode } from "react";
import "./index.css";

const Tag = ({ children }: { children: ReactNode }) => {
  return <span className="tag">{children}</span>;
};

export default Tag;