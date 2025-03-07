import { splitBlock } from "../block";
import type { PermissionLevel, ValueInfo } from "./type";

export function parseInnerValues(file: string): ValueInfo[] {
	const splittedBlock = splitBlock({ file, startsWith: "define_values!" });
	return splittedBlock.map(({ wiki, declaration, demo, extra }) => {
		const m = declaration.match(
			/\{"\$\{(\w+)}",[\w.()]+\(\),PermissionLevel::(\w+)},?/,
		);
		if (m) {
			return {
				name: m[1],
				level: m[2] as PermissionLevel,
				wiki,
				demo,
				demoValue: extra,
			};
		}
		throw new Error(
			`Error:Failed to parse value declaration line '${declaration}'`,
		);
	});
}
