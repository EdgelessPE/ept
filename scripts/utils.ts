import path from "path";

export function parseFilePath(rawPath:string){
  if (rawPath.startsWith("@/")) {
    rawPath = rawPath.replace("@/", path.join(__dirname, "../src/"));
  }
  return rawPath
}