-- Add delivery_notes column so The Statistician can attach a message/summary
-- when delivering processed data back to the original requester (Step 4 of the
-- data request workflow as defined in OFFICIAL CASE FILE/RUSA_Case_File_Parsed.docx).
ALTER TABLE data_requests
    ADD COLUMN IF NOT EXISTS delivery_notes TEXT;
