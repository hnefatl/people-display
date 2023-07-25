use secstr::SecStr;

const PASSWORD_METADATA_KEY: &'static str = "password-bin";

#[derive(Clone)]
pub struct AddPassword {
    password: SecStr,
}
impl AddPassword {
    pub fn new(password: SecStr) -> Self {
        AddPassword { password }
    }
}

impl tonic::service::Interceptor for AddPassword {
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> Result<tonic::Request<()>, tonic::Status> {
        let mut password_metadata =
            tonic::metadata::BinaryMetadataValue::from_bytes(self.password.unsecure());
        password_metadata.set_sensitive(true);
        request
            .metadata_mut()
            .insert_bin(PASSWORD_METADATA_KEY, password_metadata);
        Ok(request)
    }
}

#[derive(Clone)]
pub struct CheckPassword {
    password: SecStr,
}
impl CheckPassword {
    pub fn new(password: SecStr) -> Self {
        CheckPassword { password }
    }
}

impl tonic::service::Interceptor for CheckPassword {
    fn call(&mut self, request: tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status> {
        let metadata = request
            .metadata()
            .get_bin(PASSWORD_METADATA_KEY)
            .ok_or(tonic::Status::unauthenticated("No password provided."))?;
        let received_password_bytes = metadata.to_bytes().map_err(|e| {
            tonic::Status::invalid_argument(format!("Invalid password provided: {e}"))
        })?;

        if secstr::SecStr::new(received_password_bytes.to_vec()) != self.password {
            return Err(tonic::Status::unauthenticated("Password doesn't match."));
        }
        Ok(request)
    }
}
