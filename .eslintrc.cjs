module.exports = {
    "env": {
        "browser": true,
        "es2021": true
    },
    "extends": ["love","prettier"],
    "overrides": [
        {
            "env": {
                "node": true
            },
            "files": [
                ".eslintrc.{js,cjs}"
            ],
            "parserOptions": {
                "sourceType": "script"
            }
        }
    ],
    "parserOptions": {
        "ecmaVersion": "latest",
        "sourceType": "module"
    },
    "plugins": ["prettier"],
    "rules": {
        "prettier/prettier": "error",
        "@typescript-eslint/explicit-function-return-type": "off",
        "@typescript-eslint/strict-boolean-expressions":"off"
    }
}
