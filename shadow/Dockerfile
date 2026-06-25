FROM python:latest AS base
SHELL [ "/usr/bin/env", "bash", "-euo", "pipefail", "-c" ]
RUN --mount=type=cache,target=/root/.cache/pip \
    pip install requests pylint

FROM base AS lint
RUN mkdir e2e_output
COPY --link . .
# Here we disable some warnings since they are TOO strict:
# - C0303: Trailing whitespace (trailing-whitespace)
# - C0114: Missing module docstring (missing-module-docstring)
# - C0115: Missing class docstring (missing-class-docstring)
# - C0116: Missing function or method docstring (missing-function-docstring)
RUN <<EOF
pylint --disable=C0114,C0115,C0116,C0303 *.py \
|& tee e2e_output/pylint.log                  \
|| tee e2e_output/FAIL <<< LINT
EOF

FROM lint AS tests
RUN <<EOF
python run_tests.py        \
|& tee e2e_output/tests.log \
|| tee -a e2e_output/FAIL <<< TESTS
EOF

FROM scratch AS output
COPY --link --from=tests /e2e_output/ .
