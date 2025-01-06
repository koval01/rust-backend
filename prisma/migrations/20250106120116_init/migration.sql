-- CreateEnum
CREATE TYPE "LessonStatus" AS ENUM ('PENDING', 'COMPLETED', 'SKIPPED');

-- CreateEnum
CREATE TYPE "Level" AS ENUM ('A1', 'A2', 'B1', 'B2', 'C1', 'C2');

-- CreateTable
CREATE TABLE "User" (
    "id" BIGINT NOT NULL,
    "first_name" VARCHAR(255) NOT NULL,
    "last_name" VARCHAR(255),
    "username" VARCHAR(33),
    "language_code" VARCHAR(2) NOT NULL,
    "allows_write_to_pm" BOOLEAN NOT NULL DEFAULT false,
    "photo_url" VARCHAR(2048),
    "created_at" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMP(3) NOT NULL,

    CONSTRAINT "User_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "Lesson" (
    "id" UUID NOT NULL,
    "lesson_data" JSONB NOT NULL,
    "studied_lang" VARCHAR(2) NOT NULL,
    "lesson_lang" VARCHAR(2) NOT NULL,
    "level" "Level" NOT NULL,
    "created_at" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT "Lesson_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "UserLesson" (
    "id" UUID NOT NULL,
    "userId" BIGINT NOT NULL,
    "lessonId" UUID NOT NULL,
    "score" INTEGER NOT NULL DEFAULT 0,
    "status" "LessonStatus" NOT NULL DEFAULT 'PENDING',
    "completed_at" TIMESTAMP(3),
    "next_available" TIMESTAMP(3),

    CONSTRAINT "UserLesson_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "UserStats" (
    "userId" BIGINT NOT NULL,
    "total_score" INTEGER NOT NULL DEFAULT 0,
    "total_lessons" INTEGER NOT NULL DEFAULT 0,
    "updated_at" TIMESTAMP(3) NOT NULL,

    CONSTRAINT "UserStats_pkey" PRIMARY KEY ("userId")
);

-- CreateIndex
CREATE UNIQUE INDEX "User_username_key" ON "User"("username");

-- CreateIndex
CREATE INDEX "User_username_idx" ON "User"("username");

-- CreateIndex
CREATE INDEX "User_id_idx" ON "User"("id");

-- CreateIndex
CREATE INDEX "Lesson_studied_lang_lesson_lang_level_idx" ON "Lesson"("studied_lang", "lesson_lang", "level");

-- CreateIndex
CREATE INDEX "Lesson_id_idx" ON "Lesson"("id");

-- CreateIndex
CREATE INDEX "UserLesson_id_idx" ON "UserLesson"("id");

-- CreateIndex
CREATE INDEX "UserLesson_next_available_idx" ON "UserLesson"("next_available");

-- CreateIndex
CREATE UNIQUE INDEX "UserLesson_userId_lessonId_key" ON "UserLesson"("userId", "lessonId");

-- AddForeignKey
ALTER TABLE "UserLesson" ADD CONSTRAINT "UserLesson_userId_fkey" FOREIGN KEY ("userId") REFERENCES "User"("id") ON DELETE CASCADE ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "UserLesson" ADD CONSTRAINT "UserLesson_lessonId_fkey" FOREIGN KEY ("lessonId") REFERENCES "Lesson"("id") ON DELETE CASCADE ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "UserStats" ADD CONSTRAINT "UserStats_userId_fkey" FOREIGN KEY ("userId") REFERENCES "User"("id") ON DELETE CASCADE ON UPDATE CASCADE;

-- CreateFunction
CREATE FUNCTION set_next_available()
    RETURNS TRIGGER AS $$
BEGIN
    NEW.next_available := NOW() + INTERVAL '3 days';
RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- CreateFunction
CREATE FUNCTION create_user_stats()
    RETURNS TRIGGER AS $$
BEGIN
INSERT INTO "UserStats" ("userId")
VALUES (NEW.id);
RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- CreateTrigger
CREATE TRIGGER after_lesson_completion
    AFTER UPDATE ON "UserLesson"
    FOR EACH ROW
    WHEN (NEW.status = 'COMPLETED')
    EXECUTE FUNCTION set_next_available();

-- CreateTrigger
CREATE TRIGGER after_user_insert
    AFTER INSERT ON "User"
    FOR EACH ROW
    EXECUTE FUNCTION create_user_stats();
