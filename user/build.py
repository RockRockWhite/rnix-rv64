import os

base_address = 0x80400000
step = 0x20000
linker = "src/linker.ld"
app_id = 0

apps = os.listdir("src/bin")

# 按编号顺序排序
apps.sort()

for app in apps:
    # filename without extension
    app = app[: app.find(".")]
    print(app)

    lines = []
    lines_before = []

    # open linker file
    with open(linker, "r") as f:
        for line in f.readlines():
            lines_before.append(line)

            # modify base address
            line = line.replace(hex(base_address),
                                hex(base_address + step * app_id))
            lines.append(line)

    # write new linker file
    with open(linker, "w+") as f:
        f.writelines(lines)

    # cargo build base on the new linker file
    os.system("cargo build --bin %s --release" % app)

    print(
        "[build.py] application %s start with address %s"
        % (app, hex(base_address + step * app_id))
    )

    # write back the old linker file
    with open(linker, "w+") as f:
        f.writelines(lines_before)

    app_id = app_id + 1
