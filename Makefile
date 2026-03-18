.PHONY: download-node download-nssm frontend build-linux build-macos clean

# 下载所有平台 Node.js 归档
download-node:
	docker compose run --rm download-node

# 下载 NSSM (Windows 服务管理)
download-nssm:
	bash scripts/download-nssm.sh

# 容器内构建前端
frontend:
	docker compose run --rm frontend

# 容器内完整 Linux 构建 (.deb + .AppImage)
build-linux:
	docker compose build build-linux

# 本机 macOS 构建 (需要 Xcode CLT + Rust)
build-macos: download-node
	cd apps/installer && pnpm install && npx tauri build

# CI 全平台构建 (推送 tag 触发)
# git tag v0.1.0 && git push origin v0.1.0

clean:
	rm -rf apps/installer/src-tauri/target
	rm -rf apps/installer/dist
	rm -rf apps/installer/node_modules
	rm -f apps/installer/src-tauri/resources/node-*
	rm -f apps/installer/src-tauri/resources/nssm.exe
