-- CreateTable
CREATE TABLE "User" (
    "id" BIGINT NOT NULL,
    "first_name" TEXT NOT NULL,
    "last_name" TEXT,
    "username" TEXT,
    "language_code" TEXT NOT NULL,
    "allows_write_to_pm" BOOLEAN NOT NULL,
    "photo_url" TEXT,

    CONSTRAINT "User_pkey" PRIMARY KEY ("id")
);

-- CreateIndex
CREATE UNIQUE INDEX "User_username_key" ON "User"("username");

-- CreateIndex
CREATE INDEX "User_username_idx" ON "User"("username");

-- CreateIndex
CREATE INDEX "User_id_idx" ON "User"("id");
