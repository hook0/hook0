import { JestConfigWithTsJest } from "ts-jest";

const config: JestConfigWithTsJest = {
  preset: "ts-jest",
  testEnvironment: "node",
  transform: {
    "^.+\\.tsx?$": ["ts-jest", { tsconfig: "tsconfig.json" }],
  },
  testMatch: ["**/tests/**/*.test.ts"], // Exécute uniquement les fichiers de tests
  moduleFileExtensions: ["ts", "js"],
  clearMocks: true, // Nettoie les mocks après chaque test
};

export default config;
